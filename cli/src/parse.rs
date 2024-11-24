//! Source file parsing.
use anyhow::Context;
use crossbeam::channel::bounded;
use ignore::{DirEntry, WalkBuilder, WalkState};
use log::error;
use std::{
    collections::{hash_map::Entry, BTreeMap, HashMap},
    ops::Not,
    path::PathBuf,
    thread,
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
    out_file_name: String,
    /// The crate name the source file belongs to.
    crate_name: CrateName,
}

fn mk_parse_input(
    multi_file: bool,
    language_type: SupportedLanguage,
    dir_entry: DirEntry,
) -> Option<ParserInput> {
    let crate_name = if multi_file {
        CrateName::find_crate_name(dir_entry.path())?
    } else {
        SINGLE_FILE_CRATE_NAME
    };
    let file_path = dir_entry.path().to_path_buf();
    let out_file_name = output_file_name(language_type, &crate_name);
    Some(ParserInput {
        file_path,
        out_file_name,
        crate_name,
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

pub fn parse_input(
    parse_context: &ParseContext,
    parse_input: ParserInput,
) -> anyhow::Result<Option<ParsedData>> {
    let input_file = parse_input
        .file_path
        .to_str()
        .map(ToOwned::to_owned)
        .unwrap_or_default();

    let parse_file_context = ParseFileContext {
        source_code: std::fs::read_to_string(&parse_input.file_path)
            .with_context(|| format!("Failed to read input: {input_file}"))?,
        crate_name: parse_input.crate_name.clone(),
        file_name: parse_input.out_file_name,
        file_path: parse_input.file_path,
    };

    let parsed_result = typeshare_core::parser::parse(parse_context, parse_file_context)
        .with_context(|| format!("Failed to parse: {input_file}"))?;

    Ok(parsed_result)
}

fn parse_dir_entry(
    parse_context: &ParseContext,
    language_type: SupportedLanguage,
    dir_entry: DirEntry,
) -> Option<ParsedData> {
    dir_entry.path().is_dir().not().then_some(())?;
    let input = mk_parse_input(parse_context.multi_file, language_type, dir_entry)?;
    match parse_input(parse_context, input) {
        Ok(parsed_data) => parsed_data,
        Err(err) => {
            error!("Failed to parse {err}");
            None
        }
    }
}

/// Use parallel builder to walk all source directories concurrently.
pub fn parallel_parse(
    parse_context: &ParseContext,
    walker_builder: WalkBuilder,
    language_type: SupportedLanguage,
) -> BTreeMap<CrateName, ParsedData> {
    let (tx, rx) = bounded::<ParsedData>(100);

    let collector_thread = thread::spawn(move || {
        let mut crate_parsed_data: BTreeMap<CrateName, ParsedData> = BTreeMap::new();

        for parsed_data in rx {
            let crate_name = parsed_data.crate_name.clone();
            // Append each yielded parsed data by its respective crate.
            *crate_parsed_data.entry(crate_name).or_default() += parsed_data;
        }

        crate_parsed_data
    });

    walker_builder.build_parallel().run(|| {
        let tx = tx.clone();

        Box::new(move |result| match result {
            Ok(dir_entry) => {
                if let Some(result) = parse_dir_entry(parse_context, language_type, dir_entry) {
                    tx.send(result).unwrap();
                }
                WalkState::Continue
            }
            Err(e) => {
                error!("Traversing directories failed: {e}");
                WalkState::Quit
            }
        })
    });

    drop(tx);
    collector_thread.join().unwrap()
}
