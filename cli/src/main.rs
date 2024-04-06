//! This is the command line tool for Typeshare. It is used to generate source files in other
//! languages based on Rust code.

use clap::{command, Arg, ArgMatches, Command};
use config::Config;
use ignore::{overrides::OverrideBuilder, types::TypesBuilder, WalkBuilder};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    fs,
    ops::Not,
    path::Path,
};
#[cfg(feature = "go")]
use typeshare_core::language::Go;
use typeshare_core::{
    language::{GenericConstraints, Kotlin, Language, Scala, SupportedLanguage, Swift, TypeScript},
    parser::ParsedData,
};

mod config;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const ARG_TYPE: &str = "TYPE";
const ARG_SWIFT_PREFIX: &str = "SWIFTPREFIX";
const ARG_KOTLIN_PREFIX: &str = "KOTLINPREFIX";
const ARG_JAVA_PACKAGE: &str = "JAVAPACKAGE";
const ARG_MODULE_NAME: &str = "MODULENAME";
const ARG_SCALA_PACKAGE: &str = "SCALAPACKAGE";
const ARG_SCALA_MODULE_NAME: &str = "SCALAMODULENAME";
#[cfg(feature = "go")]
const ARG_GO_PACKAGE: &str = "GOPACKAGE";
const ARG_CONFIG_FILE_NAME: &str = "CONFIGFILENAME";
const ARG_GENERATE_CONFIG: &str = "generate-config-file";
const ARG_OUTPUT_FOLDER: &str = "output-folder";
const ARG_FOLLOW_LINKS: &str = "follow-links";

#[cfg(feature = "go")]
const AVAILABLE_LANGUAGES: [&str; 5] = ["kotlin", "scala", "swift", "typescript", "go"];

#[cfg(not(feature = "go"))]
const AVAILABLE_LANGUAGES: [&str; 4] = ["kotlin", "scala", "swift", "typescript"];

fn build_command() -> Command<'static> {
    command!("typeshare")
        .version(VERSION)
        .args_conflicts_with_subcommands(true)
        .subcommand_negates_reqs(true)
        .subcommand(
            Command::new("completions")
                .about("Generate shell completions")
                .arg(
                    Arg::new("shell")
                        .value_name("SHELL")
                        .help("The shell to generate the completions for")
                        .required(true)
                        .possible_values(clap_complete_command::Shell::possible_values()),
                ),
        )
        .arg(
            Arg::new(ARG_TYPE)
                .short('l')
                .long("lang")
                .help("Language of generated types")
                .takes_value(true)
                .possible_values(AVAILABLE_LANGUAGES)
                .required_unless(ARG_GENERATE_CONFIG),
        )
        .arg(
            Arg::new(ARG_SWIFT_PREFIX)
                .short('s')
                .long("swift-prefix")
                .help("Prefix for generated Swift types")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new(ARG_KOTLIN_PREFIX)
                .short('k')
                .long("kotlin-prefix")
                .help("Prefix for generated Kotlin types")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new(ARG_JAVA_PACKAGE)
                .short('j')
                .long("java-package")
                .help("JAVA package name")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new(ARG_MODULE_NAME)
                .short('m')
                .long("module-name")
                .help("Kotlin serializer module name")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new(ARG_SCALA_PACKAGE)
                .long("scala-package")
                .help("Scala package name")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new(ARG_SCALA_MODULE_NAME)
                .long("scala-module-name")
                .help("Scala serializer module name")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new(ARG_CONFIG_FILE_NAME)
                .short('c')
                .long("config-file")
                .help("Configuration file for typeshare")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new(ARG_GENERATE_CONFIG)
                .short('g')
                .long("generate-config-file")
                .help("Generates a configuration file based on the other options specified. The file will be written to typeshare.toml by default or to the file path specified by the --config-file option.")
                .takes_value(false)
                .required(false),
        )
        .arg(
            Arg::new(ARG_OUTPUT_FOLDER)
                .short('o')
                .long("output-folder")
                .help("Folder to write output to. mtime will be preserved if the file contents don't change")
                .required_unless(ARG_GENERATE_CONFIG)
                .takes_value(true)
                .long(ARG_OUTPUT_FOLDER)
        )
        .arg(
            Arg::new(ARG_FOLLOW_LINKS)
            .short('L')
            .long("follow-links")
            .help("Follow symbolic links to directories instead of ignoring them.")
            .takes_value(false)
            .required(false)
        )
        .arg(
            Arg::new("directories")
                .help("Directories within which to recursively find and process rust files")
                .required_unless(ARG_GENERATE_CONFIG)
                .min_values(1),
        )
}

fn main() -> Result<(), ()> {
    #[allow(unused_mut)]
    let mut command = build_command();

    #[cfg(feature = "go")]
    {
        command = command.arg(
            Arg::new(ARG_GO_PACKAGE)
                .long("go-package")
                .help("Go package name")
                .takes_value(true)
                .required_if(ARG_TYPE, "go"),
        );
    }

    let options = command.get_matches();

    if let Some(options) = options.subcommand_matches("completions") {
        if let Ok(shell) = options.value_of_t::<clap_complete_command::Shell>("shell") {
            let mut command = build_command();
            shell.generate(&mut command, &mut std::io::stdout());
        }
        return Err(());
    }

    let config_file = options.value_of(ARG_CONFIG_FILE_NAME);
    let config = config::load_config(config_file).unwrap_or_else(|error| {
        panic!("Unable to read configuration file due to error: {}", error);
    });
    let config = override_configuration(config, &options);

    if options.is_present(ARG_GENERATE_CONFIG) {
        let config = override_configuration(Config::default(), &options);
        let file_path = options.value_of(ARG_CONFIG_FILE_NAME);
        config::store_config(&config, file_path)
            .unwrap_or_else(|e| panic!("Failed to create new config file due to: {}", e));
        return Err(());
    }

    let mut directories = options.values_of("directories").unwrap();
    let language_type = options
        .value_of(ARG_TYPE)
        .map(|lang| lang.parse().ok())
        .and_then(|parsed| parsed);

    let mut lang: Box<dyn Language> = match language_type {
        Some(SupportedLanguage::Swift) => Box::new(Swift {
            prefix: config.swift.prefix,
            type_mappings: config.swift.type_mappings,
            default_decorators: config.swift.default_decorators,
            default_generic_constraints: GenericConstraints::from_config(
                config.swift.default_generic_constraints,
            ),
            ..Default::default()
        }),
        Some(SupportedLanguage::Kotlin) => Box::new(Kotlin {
            package: config.kotlin.package,
            module_name: config.kotlin.module_name,
            prefix: config.kotlin.prefix,
            type_mappings: config.kotlin.type_mappings,
            ..Default::default()
        }),
        Some(SupportedLanguage::Scala) => Box::new(Scala {
            package: config.scala.package,
            module_name: config.scala.module_name,
            type_mappings: config.scala.type_mappings,
            ..Default::default()
        }),
        Some(SupportedLanguage::TypeScript) => Box::new(TypeScript {
            type_mappings: config.typescript.type_mappings,
            ..Default::default()
        }),
        #[cfg(feature = "go")]
        Some(SupportedLanguage::Go) => Box::new(Go {
            package: config.go.package,
            type_mappings: config.go.type_mappings,
            uppercase_acronyms: config.go.uppercase_acronyms,
            ..Default::default()
        }),
        #[cfg(not(feature = "go"))]
        Some(SupportedLanguage::Go) => {
            panic!("go support is currently experimental and must be enabled as a feature flag for typeshare-cli")
        }
        _ => {
            panic!("argument parser didn't validate ARG_TYPE correctly");
        }
    };

    let mut types = TypesBuilder::new();
    types.add("rust", "*.rs").unwrap();
    types.select("rust");

    // This is guaranteed to always have at least one value by the clap configuration
    let first_root = directories.next().unwrap();

    let overrides = OverrideBuilder::new(first_root)
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
    walker_builder.follow_links(options.is_present(ARG_FOLLOW_LINKS));

    for root in directories {
        walker_builder.add(root);
    }

    // The walker ignores directories that are git-ignored. If you need
    // a git-ignored directory to be processed, add the specific directory to
    // the list of directories given to typeshare when it's invoked in the
    // makefiles
    let crate_parsed_data = parse_input(parser_inputs(walker_builder, language_type));

    // Collect all the types into a map of the file name they
    // belong too and the list of type names. Used for generating
    // imports in generated files.
    let import_candidates = all_types(&crate_parsed_data);

    let mut errors_encountered = false;

    for (_crate_name, parsed_data) in crate_parsed_data {
        if !errors_encountered && !parsed_data.errors.is_empty() {
            errors_encountered = true;
        }

        // Print any errors
        for error in &parsed_data.errors {
            eprintln!(
                "Failed to parse {} for crate {} {}",
                error.file_name, error.crate_name, error.error
            );
        }

        let outfile =
            Path::new(options.value_of(ARG_OUTPUT_FOLDER).unwrap()).join(&parsed_data.file_name);
        let mut generated_contents = Vec::new();
        lang.generate_types(&mut generated_contents, &import_candidates, &parsed_data)
            .expect("Couldn't generate types");
        match fs::read(&outfile) {
            Ok(buf) if buf == generated_contents => {
                // ok! don't need to do anything :)
                // avoid writing the file to leave the mtime intact
                // for tools which might use it to know when to
                // rebuild.
                println!("Skipping writing to {outfile:?} no changes");
                continue;
            }
            _ => {}
        }

        if !generated_contents.is_empty() {
            let out_dir = outfile.parent().unwrap();
            // If the output directory doesn't already exist, create it.
            if !out_dir.exists() {
                fs::create_dir_all(out_dir).expect("failed to create output directory");
            }

            fs::write(outfile, generated_contents).expect("failed to write output");
        }
    }

    if errors_encountered {
        eprint!("Errors encountered during parsing");
        Err(())
    } else {
        Ok(())
    }
}

/// Input data for parsing each source file.
struct ParserInput {
    file_path: String,
    file_name: String,
    crate_name: String,
}

fn parser_inputs(
    walker_builder: WalkBuilder,
    language_type: Option<SupportedLanguage>,
) -> Vec<ParserInput> {
    let glob_paths = walker_builder
        .build()
        .filter_map(Result::ok)
        .filter(|dir_entry| !dir_entry.path().is_dir())
        .filter_map(|dir_entry| {
            let extension = language_extension(language_type.unwrap());
            dir_entry
                .path()
                .to_str()
                .map(String::from)
                .and_then(|file_path| {
                    determine_crate_name(&file_path).map(|crate_name| {
                        let file_name = format!("{crate_name}.{extension}");
                        ParserInput {
                            file_path,
                            file_name,
                            crate_name,
                        }
                    })
                })
        })
        .collect::<Vec<_>>();
    glob_paths
}

/// Collect all the typeshared types into a mapping of crate names to typeshared types. This
/// mapping is used to lookup and generated import statements for generated files.
fn all_types(file_mappings: &HashMap<String, ParsedData>) -> HashMap<String, HashSet<String>> {
    file_mappings
        .iter()
        .map(|(crate_name, parsed_data)| (crate_name, parsed_data.type_names.clone()))
        .fold(
            HashMap::new(),
            |mut import_map: HashMap<String, HashSet<String>>, (crate_name, type_names)| {
                match import_map.entry(crate_name.clone()) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().extend(type_names);
                    }
                    Entry::Vacant(e) => {
                        e.insert(type_names);
                    }
                }
                import_map
            },
        )
}

/// Collect all the parsed sources into a mapping of crate name to parsed data.
fn parse_input(inputs: Vec<ParserInput>) -> HashMap<String, ParsedData> {
    inputs
        .into_par_iter()
        .flat_map(
            |ParserInput {
                 file_path,
                 file_name,
                 crate_name,
             }| {
                let data = std::fs::read_to_string(&file_path)
                    .unwrap_or_else(|e| panic!("failed to read file at {file_path:?}: {e}", e = e));
                let parsed_data =
                    typeshare_core::parser::parse(&data, crate_name.clone(), file_name.clone());
                match parsed_data {
                    Ok(data) => {
                        data.and_then(|d| is_parsed_data_empty(&d).not().then(|| (crate_name, d)))
                    }
                    Err(_) => panic!("{}", parsed_data.err().unwrap()),
                }
            },
        )
        .fold(
            HashMap::new,
            |mut file_maps: HashMap<String, ParsedData>, (crate_name, parsed_data)| {
                match file_maps.entry(crate_name) {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().add(parsed_data);
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(parsed_data);
                    }
                }
                file_maps
            },
        )
        .reduce(HashMap::new, |mut file_maps, mapping| {
            for (key, val) in mapping {
                match file_maps.entry(key) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().add(val);
                    }
                    Entry::Vacant(e) => {
                        e.insert(val);
                    }
                }
            }
            file_maps
        })
}

/// Overrides any configuration values with provided arguments
fn override_configuration(mut config: Config, options: &ArgMatches) -> Config {
    if let Some(swift_prefix) = options.value_of(ARG_SWIFT_PREFIX) {
        config.swift.prefix = swift_prefix.to_string();
    }

    if let Some(kotlin_prefix) = options.value_of(ARG_KOTLIN_PREFIX) {
        config.kotlin.prefix = kotlin_prefix.to_string();
    }

    if let Some(java_package) = options.value_of(ARG_JAVA_PACKAGE) {
        config.kotlin.package = java_package.to_string();
    }

    if let Some(module_name) = options.value_of(ARG_MODULE_NAME) {
        config.kotlin.module_name = module_name.to_string();
    }

    if let Some(scala_package) = options.value_of(ARG_SCALA_PACKAGE) {
        config.scala.package = scala_package.to_string();
    }

    if let Some(scala_module_name) = options.value_of(ARG_SCALA_MODULE_NAME) {
        config.scala.module_name = scala_module_name.to_string();
    }

    #[cfg(feature = "go")]
    if let Some(go_package) = options.value_of(ARG_GO_PACKAGE) {
        config.go.package = go_package.to_string();
    }

    config
}

fn language_extension(lanugage: SupportedLanguage) -> &'static str {
    match lanugage {
        SupportedLanguage::Go => "go",
        SupportedLanguage::Kotlin => "kt",
        SupportedLanguage::Scala => "scala",
        SupportedLanguage::Swift => "swift",
        SupportedLanguage::TypeScript => "ts",
    }
}

/// Extract the crate name from a give path.
fn determine_crate_name(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);
    let mut crate_finder = path.iter().rev().skip_while(|p| *p != "src");
    crate_finder.next();
    crate_finder
        .next()
        .and_then(|s| s.to_str())
        .map(file_name_to_crate_name)
}

/// Check if we have not parsed any relavent typehsared types.
fn is_parsed_data_empty(parsed_data: &ParsedData) -> bool {
    parsed_data.enums.is_empty()
        && parsed_data.aliases.is_empty()
        && parsed_data.structs.is_empty()
        && parsed_data.errors.is_empty()
}

/// Convert a folder crate name to a source crate name.
fn file_name_to_crate_name(file_name: &str) -> String {
    file_name.replace('-', "_")
}

#[cfg(test)]
mod test {
    use crate::determine_crate_name;

    #[test]
    fn test_crate_name() {
        let path = "/some/path/to/projects/core/foundation/op-proxy/src/android.rs";
        assert_eq!(Some("op_proxy"), determine_crate_name(path).as_deref());
    }
}
