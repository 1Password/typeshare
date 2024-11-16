//! Source file parsing.
use anyhow::Context;
use ignore::WalkBuilder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    collections::{hash_map::Entry, BTreeMap, HashMap},
    path::PathBuf,
};
use typeshare_core::{
    context::{ParseContext, ParseFileContext},
    language::{CrateName, CrateTypes, SupportedLanguage, SINGLE_FILE_CRATE_NAME},
    parser::ParsedData,
    RenameExt,
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
    multi_file: bool,
) -> impl Iterator<Item = ParserInput> {
    walker_builder
        .build()
        .filter_map(Result::ok)
        .filter(|dir_entry| !dir_entry.path().is_dir())
        .filter_map(move |dir_entry| {
            let crate_name = if multi_file {
                CrateName::find_crate_name(dir_entry.path())?
            } else {
                SINGLE_FILE_CRATE_NAME
            };
            let file_path = dir_entry.path().to_path_buf();
            let file_name = output_file_name(language_type, &crate_name);
            Some(ParserInput {
                file_path,
                file_name,
                crate_name,
            })
        })
}

/// The output file name to write to.
fn output_file_name(language_type: SupportedLanguage, crate_name: &CrateName) -> String {
    let extension = language_type.language_extension();

    let snake_case = || format!("{crate_name}.{extension}");
    let pascal_case = || format!("{}.{extension}", crate_name.to_string().to_pascal_case());

    match language_type {
        SupportedLanguage::Go => snake_case(),
        SupportedLanguage::Kotlin => snake_case(),
        SupportedLanguage::Scala => snake_case(),
        SupportedLanguage::Swift => pascal_case(),
        SupportedLanguage::TypeScript => snake_case(),
    }
}

/// Collect all the typeshared types into a mapping of crate names to typeshared types. This
/// mapping is used to lookup and generated import statements for generated files.
pub fn all_types(file_mappings: &BTreeMap<CrateName, ParsedData>) -> CrateTypes {
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
    inputs: impl ParallelIterator<Item = ParserInput>,
    parse_context: &ParseContext,
) -> anyhow::Result<BTreeMap<CrateName, ParsedData>> {
    inputs
        .into_par_iter()
        .try_fold(
            BTreeMap::new,
            |mut parsed_crates: BTreeMap<CrateName, ParsedData>,
             ParserInput {
                 file_path,
                 file_name,
                 crate_name,
             }| {
                let parse_file_context = ParseFileContext {
                    source_code: std::fs::read_to_string(&file_path)
                        .with_context(|| format!("Failed to read input: {file_name}"))?,
                    crate_name: crate_name.clone(),
                    file_name: file_name.clone(),
                    file_path,
                };

                let parsed_result =
                    typeshare_core::parser::parse(parse_context, parse_file_context)
                        .with_context(|| format!("Failed to parse: {file_name}"))?;

                if let Some(parsed_data) = parsed_result {
                    parsed_crates
                        .entry(crate_name)
                        .or_default()
                        .add(parsed_data);
                }

                Ok(parsed_crates)
            },
        )
        .try_reduce(BTreeMap::new, |mut file_maps, parsed_crates| {
            for (crate_name, parsed_data) in parsed_crates {
                file_maps.entry(crate_name).or_default().add(parsed_data);
            }
            Ok(file_maps)
        })
}
