//! This is the command line tool for Typeshare. It is used to generate source files in other
//! languages based on Rust code.
//!

mod args;
mod config;
mod parse;
mod writer;

use std::{
    collections::{BTreeMap, HashMap},
    io,
    path::Path,
};

use anyhow::{anyhow, Context};
use clap::{CommandFactory, Parser};
use clap_complete::aot::generate;
use flexi_logger::AdaptiveFormat;
use ignore::{overrides::OverrideBuilder, types::TypesBuilder, WalkBuilder};
use log::{error, info};
use rayon::iter::ParallelBridge;
#[cfg(feature = "go")]
use typeshare_core::language::Go;
use typeshare_core::{
    context::ParseContext,
    language::{
        CrateName, GenericConstraints, Kotlin, Language, Scala, SupportedLanguage, Swift,
        TypeScript,
    },
    parser::ParsedData,
};

use crate::{
    args::{Args, Command},
    config::Config,
    parse::{all_types, parse_input, parser_inputs},
    writer::{write_generated, Output},
};

fn main() -> anyhow::Result<()> {
    flexi_logger::Logger::try_with_env_or_str("info")?
        .adaptive_format_for_stderr(AdaptiveFormat::Detailed)
        .adaptive_format_for_stdout(AdaptiveFormat::Detailed)
        .start()?;

    let options = Args::parse();

    info!("typeshare started generating types");

    if let Some(options) = options.subcommand {
        match options {
            Command::Completions { shell } => {
                let mut cmd = Args::command();
                let bin_name = cmd.get_name().to_string();
                generate(shell, &mut cmd, bin_name, &mut io::stdout());
            }
        }

        return Ok(());
    }

    // Note that this can be `None`; the relevant functions handle this case
    // on their own.
    let config_file = options.config_file.as_deref();

    if options.output.generate_config {
        let config = override_configuration(Config::default(), &options)?;
        config::store_config(&config, config_file).context("Failed to create new config file")?;
        return Ok(());
    }

    let config = config::load_config(config_file).context("Unable to read configuration file")?;
    let config = override_configuration(config, &options)?;

    let directories = options.directories.as_slice();

    info!("Using directories: {directories:?}");

    let language_type = match options.language {
        None => panic!("no language specified; `clap` should have guaranteed its presence"),
        Some(language) => match language {
            args::AvailableLanguage::Kotlin => SupportedLanguage::Kotlin,
            args::AvailableLanguage::Scala => SupportedLanguage::Scala,
            args::AvailableLanguage::Swift => SupportedLanguage::Swift,
            args::AvailableLanguage::Typescript => SupportedLanguage::TypeScript,
            #[cfg(feature = "go")]
            args::AvailableLanguage::Go => SupportedLanguage::Go,
        },
    };

    let mut types = TypesBuilder::new();
    types
        .add("rust", "*.rs")
        .context("Failed to add rust type extensions")?;
    types.select("rust");

    // This is guaranteed to always have at least one value by the clap configuration
    let first_root = directories
        .first()
        .expect("directories is empty; this shouldn't be possible");

    let overrides = OverrideBuilder::new(first_root)
        // Don't process files inside of tools/typeshare/
        .add("!**/tools/typeshare/**")
        .context("Failed to parse override")?
        .build()
        .context("Failed to build override")?;

    let mut walker_builder = WalkBuilder::new(first_root);
    // Sort walker output for deterministic output across platforms
    walker_builder
        .sort_by_file_path(Path::cmp)
        .types(types.build().context("Failed to build types")?)
        .overrides(overrides)
        .follow_links(options.follow_links);

    for root in directories.iter().skip(1) {
        walker_builder.add(root);
    }

    let destination = if let Some(ref file) = options.output.file {
        Output::File(file)
    } else if let Some(ref folder) = options.output.folder {
        Output::Folder(folder)
    } else {
        panic!(
            "Got neither a file nor a folder to output to; this indicates a
            bug in typeshare, since `clap` is supposed to prevent this"
        )
    };

    let multi_file = matches!(destination, Output::Folder(_));
    let target_os = config.target_os.clone();
    let mut lang = language(language_type, config, multi_file);

    let parse_context = ParseContext {
        ignored_types: lang.ignored_reference_types(),
        multi_file,
        target_os,
    };

    // The walker ignores directories that are git-ignored. If you need
    // a git-ignored directory to be processed, add the specific directory to
    // the list of directories given to typeshare when it's invoked in the
    // makefiles
    // TODO: The `ignore` walker supports parallel walking. We should use this
    // and implement a `ParallelVisitor` that builds up the mapping of parsed
    // data. That way both walking and parsing are in parallel.
    // https://docs.rs/ignore/latest/ignore/struct.WalkParallel.html
    let crate_parsed_data = parse_input(
        parser_inputs(walker_builder, language_type, multi_file).par_bridge(),
        &parse_context,
    )?;

    // Collect all the types into a map of the file name they
    // belong too and the list of type names. Used for generating
    // imports in generated files.
    let import_candidates = if multi_file {
        all_types(&crate_parsed_data)
    } else {
        HashMap::new()
    };

    check_parse_errors(&crate_parsed_data)?;

    info!("typeshare started writing generated types");

    write_generated(
        destination,
        lang.as_mut(),
        crate_parsed_data,
        import_candidates,
    )?;

    info!("typeshare finished generating types");
    Ok(())
}

/// Get the language trait impl for the given supported language and configuration.
fn language(
    language_type: SupportedLanguage,
    config: Config,
    multi_file: bool,
) -> Box<dyn Language> {
    match language_type {
        SupportedLanguage::Swift => Box::new(Swift {
            prefix: config.swift.prefix,
            type_mappings: config.swift.type_mappings,
            default_decorators: config.swift.default_decorators,
            default_generic_constraints: GenericConstraints::from_config(
                config.swift.default_generic_constraints,
            ),
            multi_file,
            codablevoid_constraints: config.swift.codablevoid_constraints,
            ..Default::default()
        }),
        SupportedLanguage::Kotlin => Box::new(Kotlin {
            package: config.kotlin.package,
            module_name: config.kotlin.module_name,
            prefix: config.kotlin.prefix,
            type_mappings: config.kotlin.type_mappings,
            ..Default::default()
        }),
        SupportedLanguage::Scala => Box::new(Scala {
            package: config.scala.package,
            module_name: config.scala.module_name,
            type_mappings: config.scala.type_mappings,
            ..Default::default()
        }),
        SupportedLanguage::TypeScript => Box::new(TypeScript {
            type_mappings: config.typescript.type_mappings,
            ..Default::default()
        }),
        #[cfg(feature = "go")]
        SupportedLanguage::Go => Box::new(Go {
            package: config.go.package,
            type_mappings: config.go.type_mappings,
            uppercase_acronyms: config.go.uppercase_acronyms,
            no_pointer_slice: config.go.no_pointer_slice,
            ..Default::default()
        }),
        #[cfg(not(feature = "go"))]
        SupportedLanguage::Go => {
            panic!("go support is currently experimental and must be enabled as a feature flag for typeshare-cli")
        }
    }
}

/// Overrides any configuration values with provided arguments
fn override_configuration(mut config: Config, options: &Args) -> anyhow::Result<Config> {
    if let Some(swift_prefix) = options.swift_prefix.as_ref() {
        config.swift.prefix = swift_prefix.clone();
    }

    if let Some(kotlin_prefix) = options.kotlin_prefix.as_ref() {
        config.kotlin.prefix = kotlin_prefix.clone();
    }

    if let Some(java_package) = options.java_package.as_ref() {
        config.kotlin.package = java_package.clone();
    }

    if let Some(module_name) = options.kotlin_module_name.as_ref() {
        config.kotlin.module_name = module_name.to_string();
    }

    if let Some(scala_package) = options.scala_package.as_ref() {
        config.scala.package = scala_package.clone();
    }

    if let Some(scala_module_name) = options.scala_module_name.as_ref() {
        config.scala.module_name = scala_module_name.to_string();
    }

    #[cfg(feature = "go")]
    {
        if let Some(go_package) = options.go_package.as_ref() {
            config.go.package = go_package.to_string();
        }

        if matches!(options.language, Some(args::AvailableLanguage::Go)) {
            anyhow::ensure!(
                    !config.go.package.is_empty(),
                   "Please provide a package name in the typeshare.toml or using --go-package <package name>"
                );
        }
    }

    config.target_os = options.target_os.as_deref().unwrap_or_default().to_vec();

    Ok(config)
}

/// Prints out all parsing errors if any and returns Err.
fn check_parse_errors(parsed_crates: &BTreeMap<CrateName, ParsedData>) -> anyhow::Result<()> {
    let mut errors_encountered = false;
    for data in parsed_crates
        .values()
        .filter(|parsed_data| !parsed_data.errors.is_empty())
    {
        errors_encountered = true;
        for error in &data.errors {
            error!(
                "Parsing error: \"{}\" in file \"{}\"",
                error.error, error.file_name
            );
        }
    }

    if errors_encountered {
        error!("Errors encountered during parsing.");
        Err(anyhow!("Errors encountered during parsing."))
    } else {
        Ok(())
    }
}
