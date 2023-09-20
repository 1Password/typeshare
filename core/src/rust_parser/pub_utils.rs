use crate::language::{Generate, Language};
use crate::parsed_types::{Comment, ParsedData, Source};
use crate::rust_parser::ParseError;
use crate::{write_into_one, write_parse, ProcessInputError};
use ignore::overrides::OverrideBuilder;
use ignore::types::TypesBuilder;
use ignore::WalkBuilder;
use log::{debug, error, warn};
use std::collections::VecDeque;
use std::io::Write;
use std::path::Path;

/// Will process the input and write the output to the given writer.
///
/// If multiple files are generated, they will be written to the given writer
/// With Comments indicating the beginning and end of each file.
pub fn process_into_one<L: Language>(
    language: &mut L,
    data: &str,
    out: &mut impl Write,
    source: Source,
) -> Result<(), ProcessInputError<L::Error>>
where
    Comment: Generate<L>,
{
    let data = super::parse(data, source)?;
    write_into_one(language, data, out)?;
    Ok(())
}
pub fn process_directories_and_write<L: Language>(
    language: &mut L,
    mut directories: VecDeque<String>,
    out: impl AsRef<Path>,
) -> Result<(), ProcessInputError<L::Error>> {
    if directories.is_empty() {
        warn!("No directories specified. Exiting.");
        return Ok(());
    }
    let mut types = TypesBuilder::new();
    types.add("rust", "*.rs").unwrap();
    types.select("rust");
    let types = types.build()?;

    let first_root = directories.pop_front().unwrap();

    let overrides = OverrideBuilder::new(&first_root)
        // Don't process files inside of tools/typeshare/
        .add("!**/tools/typeshare/**")?
        .build()?;

    let mut walker_builder = WalkBuilder::new(first_root);

    // Sort walker output for deterministic output across platforms
    walker_builder.sort_by_file_path(|a, b| a.cmp(b));
    walker_builder.types(types);
    walker_builder.overrides(overrides);
    for root in directories {
        walker_builder.add(root);
    }

    let glob_paths = walker_builder.build();
    let mut parsed_data = ParsedData::default();

    for filepath in glob_paths {
        let filepath = match filepath {
            Ok(ok) => ok,
            Err(err) => {
                error!("Error reading file: {}", err);
                continue;
            }
        };
        if filepath.path().is_dir() {
            continue;
        }
        // TODO Improve Source generation
        let mut source = Source::default();
        source.build_from_path(&filepath.path());
        let data = std::fs::read_to_string(filepath.path())?;
        let result = super::parse(&data, source);
        match result {
            Ok(ok) => {
                debug!("Parsed file {}", filepath.path().display());
                parsed_data = parsed_data + ok;
            }
            Err(err) => {
                match err {
                    ParseError::SynError(err) => {
                        error!(
                            "Error parsing file: {:?}, span: {:?}, file: {}",
                            err,
                            err.span().source_text(),
                            filepath.path().display()
                        );
                        continue;
                    }
                    _ => {}
                }
                error!(
                    "Error parsing file: {:?} file: {}",
                    err,
                    filepath.path().display()
                );
                continue;
            }
        }
    }

    write_parse(language, parsed_data, out)?;
    Ok(())
}
pub fn process_input_and_write<L: Language>(
    language: &mut L,
    input: &str,
    out: impl AsRef<Path>,
    source: Source,
) -> Result<(), ProcessInputError<L::Error>> {
    let data = super::parse(input, source)?;
    write_parse(language, data, out)?;
    Ok(())
}
