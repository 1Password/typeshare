//! This is the command line tool for Typeshare. It is used to generate source files in other
//! languages based on Rust code.

use crate::config::{KotlinParams, ScalaParams, SwiftParams, DEFAULT_CONFIG_FILE_NAME};
use clap::{command, Arg, ArgMatches, Command, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use config::Config;
use ignore::overrides::OverrideBuilder;
use ignore::types::TypesBuilder;
use ignore::WalkBuilder;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::{fs, mem};
use typeshare_core::language::GenericConstraints;
#[cfg(feature = "go")]
use typeshare_core::language::Go;
use typeshare_core::{
    language::{Kotlin, Language, Scala, Swift, TypeScript},
    parser::ParsedData,
};

mod config;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, ValueEnum)]
pub enum LanguageOption {
    #[cfg(feature = "go")]
    Go,
    Kotlin,
    Scala,
    Swift,
    TypeScript,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[clap(long, short = 'c')]
    config_file: Option<PathBuf>,
    #[clap(subcommand)]
    subcmd: Option<Commands>,
}
#[derive(Parser)]
pub struct GenerateCommandWithConfig {
    #[clap(long, short = 'c')]
    config_file: Option<PathBuf>,
    #[clap(flatten)]
    generate: GenerateCommand,
}
#[derive(Parser)]
pub struct GenerateCommand {
    #[clap(long, short = 'l')]
    lang: LanguageOption,
    #[clap(flatten)]
    kotlin: KotlinParams,
    #[clap(flatten)]
    scala: ScalaParams,
    #[clap(flatten)]
    swift: SwiftParams,
    #[cfg(feature = "go")]
    #[clap(flatten)]
    go: config::GoParams,
    #[clap(long, short = 'o')]
    output_file: PathBuf,
    directories: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(
        about = "Initialize a typeshare.toml file in the current directory or the directory specified by -c"
    )]
    Init,
    #[clap(about = "Generate code from the Rust files in the specified directories")]
    Generate(GenerateCommand),
    #[clap(about = "Generate completion scripts for your shell")]
    Completions { generator: Option<Shell> },
}
fn main() {
    let mut command = Cli::parse();
    let (mut generate, config_path) = if let Some(subcommand) = command.subcmd {
        let config_path = if let Some(config_file) = command.config_file {
            config_file
        } else {
            PathBuf::from(DEFAULT_CONFIG_FILE_NAME)
        };
        match subcommand {
            Commands::Init => {
                if config_path.exists() {
                    println!("Config file already exists at {:?}", config_path);
                    return;
                }
                let config = Config::default();
                let config_string = toml::to_string_pretty(&config).unwrap();
                fs::write(config_path, config_string).unwrap();
                return;
            }
            Commands::Generate(generate) => (generate, config_path),
            Commands::Completions { generator } => {
                let generator = generator.unwrap_or_else(|| {
                    let option = Shell::from_env();
                    eprintln!(
                        "No shell specified. Pulling from environment. (SHELL={:?})",
                        option
                    );
                    option.unwrap()
                });
                let mut command = Cli::command();
                clap_complete::generate(
                    generator,
                    &mut command,
                    "typeshare",
                    &mut std::io::stdout(),
                );
                return;
            }
        }
    } else {
        let config1 = GenerateCommandWithConfig::parse();
        let config_path = if let Some(config_file) = command.config_file {
            config_file
        } else {
            PathBuf::from(DEFAULT_CONFIG_FILE_NAME)
        };
        (config1.generate, config_path)
    };

    let config = config::load_config(config_path).unwrap_or_else(|error| {
        panic!("Unable to read configuration file due to error: {}", error);
    });
    let config = override_configuration(config, &mut generate);

    let outfile = generate.output_file;

    let mut lang: Box<dyn Language> = match generate.lang {
        LanguageOption::Swift => Box::new(Swift {
            prefix: config.swift.prefix.unwrap_or_default(),
            type_mappings: config.swift.type_mappings,
            default_decorators: config.swift.default_decorators,
            default_generic_constraints: GenericConstraints::from_config(
                config.swift.default_generic_constraints,
            ),
            ..Default::default()
        }),
        LanguageOption::Kotlin => Box::new(Kotlin {
            package: config.kotlin.java_package.unwrap_or_default(),
            module_name: config.kotlin.kotlin_module_name.unwrap_or_default(),
            type_mappings: config.kotlin.type_mappings,
            ..Default::default()
        }),
        LanguageOption::Scala => Box::new(Scala {
            package: config.scala.scala_package.unwrap_or_default(),
            module_name: config.scala.scala_module_name.unwrap_or_default(),
            type_mappings: config.scala.type_mappings,
            ..Default::default()
        }),
        LanguageOption::TypeScript => Box::new(TypeScript {
            type_mappings: config.typescript.type_mappings,
            enum_write_method: config.typescript.enum_write_method,
            ..Default::default()
        }),
        #[cfg(feature = "go")]
        LanguageOption::Go => Box::new(Go {
            package: config.go.go_package.unwrap_or_default(),
            type_mappings: config.go.type_mappings,
            uppercase_acronyms: config.go.uppercase_acronyms,
            ..Default::default()
        }),
    };

    let mut types = TypesBuilder::new();
    types.add("rust", "*.rs").unwrap();
    types.select("rust");
    let mut directories = config.directories;
    if directories.is_empty() {
        panic!("No directories specified. Exiting.");
    }

    let first_root = directories.pop_front().unwrap();

    let overrides = OverrideBuilder::new(&first_root)
        // Don't process files inside of tools/typeshare/
        .add("!**/tools/typeshare/**")
        .expect("Failed to parse override")
        .build()
        .expect("Failed to build override");

    let mut walker_builder = WalkBuilder::new(first_root);
    // Sort walker output for deterministic output across platforms
    walker_builder.sort_by_file_path(|a, b| a.cmp(b));
    walker_builder.types(types.build().expect("Failed to build types"));
    walker_builder.overrides(overrides);

    for root in directories {
        walker_builder.add(root);
    }

    // The walker ignores directories that are git-ignored. If you need
    // a git-ignored directory to be processed, add the specific directory to
    // the list of directories given to typeshare when it's invoked in the
    // makefiles
    let glob_paths: Vec<String> = walker_builder
        .build()
        .filter_map(Result::ok)
        .filter(|dir_entry| !dir_entry.path().is_dir())
        .filter_map(|dir_entry| dir_entry.path().to_str().map(String::from))
        .collect();

    let mut generated_contents = vec![];
    let parsed_data = glob_paths
        .par_iter()
        .map(|filepath| {
            let data = std::fs::read_to_string(filepath).unwrap_or_else(|e| {
                panic!(
                    "failed to read file at {filepath:?}: {e}",
                    filepath = filepath,
                    e = e
                )
            });
            let parsed_data = typeshare_core::parser::parse(&data);
            if parsed_data.is_err() {
                panic!("{}", parsed_data.err().unwrap());
            }
            parsed_data.unwrap()
        })
        .reduce(ParsedData::default, |mut identity, other| {
            identity.add(other);
            identity
        });

    lang.generate_types(&mut generated_contents, &parsed_data)
        .expect("Couldn't generate types");

    match fs::read(&outfile) {
        Ok(buf) if buf == generated_contents => {
            // ok! don't need to do anything :)
            // avoid writing the file to leave the mtime intact
            // for tools which might use it to know when to
            // rebuild.
            return;
        }
        _ => {}
    }

    let out_dir = outfile.parent().unwrap();
    // If the output directory doesn't already exist, create it.
    if !out_dir.exists() {
        fs::create_dir_all(out_dir).expect("failed to create output directory");
    }

    fs::write(outfile, generated_contents).expect("failed to write output");
}

/// Overrides any configuration values with provided arguments
fn override_configuration(mut config: Config, options: &mut GenerateCommand) -> Config {
    if !options.directories.is_empty() {
        config.directories = VecDeque::from(mem::take(&mut options.directories));
    }

    config
}
