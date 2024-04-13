//! This is the command line tool for Typeshare. It is used to generate source files in other
//! languages based on Rust code.

use args::build_command;
use args::{
    ARG_CONFIG_FILE_NAME, ARG_FOLLOW_LINKS, ARG_GENERATE_CONFIG, ARG_JAVA_PACKAGE,
    ARG_KOTLIN_PREFIX, ARG_MODULE_NAME, ARG_SCALA_MODULE_NAME, ARG_SCALA_PACKAGE, ARG_SWIFT_PREFIX,
    ARG_TYPE,
};
use clap::ArgMatches;
use config::Config;
use ignore::{overrides::OverrideBuilder, types::TypesBuilder, WalkBuilder};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    collections::{hash_map::Entry, HashMap},
    ops::Not,
    path::PathBuf,
};
#[cfg(feature = "go")]
use typeshare_core::language::Go;
use typeshare_core::{
    language::{
        CrateName, CrateTypes, GenericConstraints, Kotlin, Language, Scala, SupportedLanguage,
        Swift, TypeScript,
    },
    parser::ParsedData,
};
use writer::write_generated;

mod args;
mod config;
mod writer;

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
        .map(|lang| lang.parse::<SupportedLanguage>().ok())
        .and_then(|parsed| parsed)
        .expect("argument parser didn't validate ARG_TYPE correctly");

    let lang = language(language_type, config);

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

    let ignored_types = lang.ignored_reference_types();

    // The walker ignores directories that are git-ignored. If you need
    // a git-ignored directory to be processed, add the specific directory to
    // the list of directories given to typeshare when it's invoked in the
    // makefiles
    let crate_parsed_data =
        parse_input(parser_inputs(walker_builder, language_type), &ignored_types);

    // Collect all the types into a map of the file name they
    // belong too and the list of type names. Used for generating
    // imports in generated files.
    let import_candidates = all_types(&crate_parsed_data);

    check_parse_errors(&crate_parsed_data)?;
    write_generated(options, lang, crate_parsed_data, import_candidates);

    Ok(())
}

fn language(language_type: SupportedLanguage, config: Config) -> Box<dyn Language> {
    match language_type {
        SupportedLanguage::Swift => Box::new(Swift {
            prefix: config.swift.prefix,
            type_mappings: config.swift.type_mappings,
            default_decorators: config.swift.default_decorators,
            default_generic_constraints: GenericConstraints::from_config(
                config.swift.default_generic_constraints,
            ),
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
            ..Default::default()
        }),
        #[cfg(not(feature = "go"))]
        SupportedLanguage::Go => {
            panic!("go support is currently experimental and must be enabled as a feature flag for typeshare-cli")
        }
    }
}

/// Input data for parsing each source file.
struct ParserInput {
    file_path: PathBuf,
    file_name: String,
    crate_name: CrateName,
}

fn parser_inputs(
    walker_builder: WalkBuilder,
    language_type: SupportedLanguage,
) -> Vec<ParserInput> {
    let glob_paths = walker_builder
        .build()
        .filter_map(Result::ok)
        .filter(|dir_entry| !dir_entry.path().is_dir())
        .filter_map(|dir_entry| {
            let extension = language_type.language_extension();
            let crate_name = CrateName::find_crate_name(dir_entry.path())?;
            let file_path = dir_entry.path().to_path_buf();
            let file_name = format!("{crate_name}.{extension}");
            Some(ParserInput {
                file_path,
                file_name,
                crate_name,
            })
        })
        .collect::<Vec<_>>();
    glob_paths
}

/// Collect all the typeshared types into a mapping of crate names to typeshared types. This
/// mapping is used to lookup and generated import statements for generated files.
fn all_types(file_mappings: &HashMap<CrateName, ParsedData>) -> CrateTypes {
    file_mappings
        .iter()
        .map(|(crate_name, parsed_data)| (crate_name, parsed_data.type_names.clone()))
        .fold(
            HashMap::new(),
            |mut import_map: CrateTypes, (crate_name, type_names)| {
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
fn parse_input(
    inputs: Vec<ParserInput>,
    ignored_types: &[String],
) -> HashMap<CrateName, ParsedData> {
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
                let parsed_data = typeshare_core::parser::parse(
                    &data,
                    crate_name.clone(),
                    file_name.clone(),
                    file_path,
                    ignored_types,
                );
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
            |mut file_maps: HashMap<CrateName, ParsedData>, (crate_name, parsed_data)| {
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

/// Check if we have not parsed any relavent typehsared types.
fn is_parsed_data_empty(parsed_data: &ParsedData) -> bool {
    parsed_data.enums.is_empty()
        && parsed_data.aliases.is_empty()
        && parsed_data.structs.is_empty()
        && parsed_data.errors.is_empty()
}

fn check_parse_errors(parsed_crates: &HashMap<CrateName, ParsedData>) -> Result<(), ()> {
    let mut errors_encountered = false;
    for data in parsed_crates
        .values()
        .filter(|parsed_data| !parsed_data.errors.is_empty())
    {
        errors_encountered = true;
        for error in &data.errors {
            eprintln!(
                "Parsing error: {} in crate {} for file {}",
                error.error, error.crate_name, error.file_name
            );
        }
    }

    if errors_encountered {
        Err(())
    } else {
        Ok(())
    }
}
