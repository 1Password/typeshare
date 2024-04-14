//! Source file parsing.
use anyhow::Context;
use ignore::WalkBuilder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    collections::{hash_map::Entry, HashMap},
    ops::Not,
    path::PathBuf,
};
use typeshare_core::{
    language::{CrateName, CrateTypes, SupportedLanguage},
    parser::ParsedData,
};

/// Input data for parsing each source file.
pub struct ParserInput {
    /// Rust source file path.
    file_path: PathBuf,
    /// File name source from crate for output.
    file_name: String,
    /// The crate name the source file belongs to.
    crate_name: CrateName,
}

/// Walk the source folder and collect all parser inputs.
pub fn parser_inputs(
    walker_builder: WalkBuilder,
    language_type: SupportedLanguage,
) -> Vec<ParserInput> {
    walker_builder
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
        .collect()
}

/// Collect all the typeshared types into a mapping of crate names to typeshared types. This
/// mapping is used to lookup and generated import statements for generated files.
pub fn all_types(file_mappings: &HashMap<CrateName, ParsedData>) -> CrateTypes {
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
pub fn parse_input(
    inputs: Vec<ParserInput>,
    ignored_types: &[String],
    multi_file: bool,
) -> anyhow::Result<HashMap<CrateName, ParsedData>> {
    inputs
        .into_par_iter()
        .try_fold(
            HashMap::new,
            |mut results: HashMap<CrateName, ParsedData>,
             ParserInput {
                 file_path,
                 file_name,
                 crate_name,
             }| {
                match std::fs::read_to_string(&file_path)
                    .context("Failed to read input")
                    .and_then(|data| {
                        typeshare_core::parser::parse(
                            &data,
                            crate_name.clone(),
                            file_name.clone(),
                            file_path,
                            ignored_types,
                            multi_file,
                        )
                        .context("Failed to parse")
                    })
                    .map(|parsed_data| {
                        parsed_data.and_then(|parsed_data| {
                            is_parsed_data_empty(&parsed_data)
                                .not()
                                .then_some((crate_name, parsed_data))
                        })
                    })? {
                    Some((crate_name, parsed_data)) => {
                        match results.entry(crate_name) {
                            Entry::Occupied(mut entry) => {
                                entry.get_mut().add(parsed_data);
                            }
                            Entry::Vacant(entry) => {
                                entry.insert(parsed_data);
                            }
                        }
                        Ok::<_, anyhow::Error>(results)
                    }
                    None => Ok(results),
                }
            },
        )
        .try_reduce(HashMap::new, |mut file_maps, mapping| {
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
            Ok(file_maps)
        })
}

/// Check if we have not parsed any relavent typehsared types.
fn is_parsed_data_empty(parsed_data: &ParsedData) -> bool {
    parsed_data.enums.is_empty()
        && parsed_data.aliases.is_empty()
        && parsed_data.structs.is_empty()
        && parsed_data.errors.is_empty()
}
