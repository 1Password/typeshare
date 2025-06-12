//! Source file parsing.
use anyhow::Context;
use crossbeam::channel::bounded;
use ignore::{DirEntry, WalkBuilder, WalkState};
use std::{
    collections::{BTreeMap, HashMap},
    mem, thread,
};
use typeshare_core::{
    context::{ParseContext, ParseFileContext},
    language::{CrateName, CrateTypes, SupportedLanguage, SINGLE_FILE_CRATE_NAME},
    parser::{ParseError, ParsedData},
    RenameExt,
};

fn parse_file_context(
    multi_file: bool,
    language_type: SupportedLanguage,
    dir_entry: &DirEntry,
) -> anyhow::Result<Option<ParseFileContext>> {
    let crate_name = if multi_file {
        let Some(crate_name) = CrateName::find_crate_name(dir_entry.path()) else {
            return Ok(None);
        };
        crate_name
    } else {
        SINGLE_FILE_CRATE_NAME
    };
    let file_path = dir_entry.path().to_path_buf();
    let out_file_name = output_file_name(language_type, &crate_name);

    let input_file = file_path
        .to_str()
        .map(ToOwned::to_owned)
        .unwrap_or_default();

    let parse_file_context = ParseFileContext {
        source_code: std::fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read input: {input_file}"))?,
        crate_name,
        file_name: out_file_name,
        file_path,
    };

    Ok(Some(parse_file_context))
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
        SupportedLanguage::Python => snake_case(),
        SupportedLanguage::C => snake_case(),
        SupportedLanguage::Cpp => snake_case(),
    }
}

/// Collect all the typeshared types into a mapping of crate names to typeshared types. This
/// mapping is used to lookup and generated import statements for generated files.
pub fn all_types(file_mappings: &mut BTreeMap<CrateName, ParsedData>) -> CrateTypes {
    file_mappings
        .iter_mut()
        .map(|(crate_name, parsed_data)| (crate_name, mem::take(&mut parsed_data.type_names)))
        .fold(
            HashMap::new(),
            |mut import_map: CrateTypes, (crate_name, type_names)| {
                import_map
                    .entry(crate_name.clone())
                    .or_default()
                    .extend(type_names);
                import_map
            },
        )
}

fn parse_dir_entry(
    parse_context: &ParseContext,
    language_type: SupportedLanguage,
    dir_entry: &DirEntry,
) -> Result<Option<ParsedData>, ParseError> {
    if dir_entry.path().is_dir() {
        return Ok(None);
    }

    let Some(parse_file_context) =
        parse_file_context(parse_context.multi_file, language_type, dir_entry)
            .map_err(|err| ParseError::IOError(err.to_string()))?
    else {
        return Ok(None);
    };

    typeshare_core::parser::parse(parse_context, parse_file_context)
}

/// Use parallel builder to walk all source directories concurrently.
pub fn parallel_parse(
    parse_context: &ParseContext,
    walker_builder: WalkBuilder,
    language_type: SupportedLanguage,
) -> anyhow::Result<BTreeMap<CrateName, ParsedData>> {
    let (tx, rx) = bounded::<anyhow::Result<ParsedData>>(100);

    let collector_thread = thread::spawn(move || {
        let mut crate_parsed_data: BTreeMap<CrateName, ParsedData> = BTreeMap::new();

        for result in rx {
            let parsed_data = result?;
            let crate_name = parsed_data.crate_name.clone();
            // Append each yielded parsed data by its respective crate.
            *crate_parsed_data.entry(crate_name).or_default() += parsed_data;
        }

        Ok(crate_parsed_data)
    });

    walker_builder.build_parallel().run(|| {
        let tx = tx.clone();

        Box::new(move |result| {
            let result = result.context("Failed traversing").and_then(|dir_entry| {
                parse_dir_entry(parse_context, language_type, &dir_entry)
                    .with_context(|| format!("Parsing failed: {:?}", dir_entry.path()))
            });
            match result {
                Ok(Some(parsed_data)) => {
                    tx.send(Ok(parsed_data)).unwrap();
                    WalkState::Continue
                }
                Ok(None) => WalkState::Continue,
                Err(err) => {
                    tx.send(Err(err)).unwrap();
                    WalkState::Quit
                }
            }
        })
    });

    drop(tx);
    collector_thread.join().unwrap()
}
