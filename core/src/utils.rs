use ignore::overrides::OverrideBuilder;
use ignore::types::TypesBuilder;
use ignore::WalkBuilder;
use std::collections::VecDeque;
use std::convert::Infallible;
use std::error::Error;
use std::{
    fs::{create_dir_all, OpenOptions},
    io,
    io::Write,
    path::Path,
};

use crate::language::Language;
pub use crate::topsort::topsort;
use log::{debug, error, info, warn};

use crate::parsed_types::Source;
use crate::parser::Parser;
use crate::{
    language::{Generate, LanguageConfig, LanguageError, WriteTypesResult},
    parsed_types::{Comment, CommentLocation, ParsedData},
    ProcessInputError,
};

/// Will process the input and write the output to the given writer.
///
/// If multiple files are generated, they will be written to the given writer
/// With Comments indicating the beginning and end of each file.
pub fn write_into_one<L: Language>(
    language: &mut L,
    data: ParsedData,
    out: &mut impl Write,
) -> Result<(), ProcessInputError<Infallible, L::Error>>
where
    Comment: Generate<L>,
{
    match language.generate_from_parse(&data)? {
        WriteTypesResult::MultiFile { files } => {
            for multi_file_item in files {
                <Comment as Generate<L>>::generate_to(
                    &Comment::new_single(
                        format!("Start of File {}", multi_file_item.name),
                        CommentLocation::FileHeader,
                    ),
                    language,
                    out,
                )?;

                out.write_all(multi_file_item.content.as_bytes())?;
                <Comment as Generate<L>>::generate_to(
                    &Comment::new_single(
                        format!("End of File {}", multi_file_item.name),
                        CommentLocation::FileHeader,
                    ),
                    language,
                    out,
                )?;
            }
        }
        WriteTypesResult::SingleFile(single) => {
            out.write_all(single.as_bytes())?;
        }
    }
    Ok(())
}
pub fn write_parse<L: Language>(
    language: &mut L,
    data: ParsedData,
    out: impl AsRef<Path>,
) -> Result<(), LanguageError<L::Error>> {
    if language.requires_multiple_files() {
        info!("Generating types to {}", out.as_ref().display());
        if out.as_ref().exists() && !out.as_ref().is_dir() {
            return Err(LanguageError::from(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "Output path {} already exists and is not a directory",
                    out.as_ref().display()
                ),
            )));
        }
        create_dir_all(out.as_ref())?;

        match language.generate_from_parse(&data)? {
            WriteTypesResult::MultiFile { files } => {
                for multi_file_item in files {
                    let write_to = out.as_ref().join(multi_file_item.name);
                    info!(
                        "Writing {} to {}",
                        multi_file_item.internal_type,
                        write_to.display()
                    );
                    let mut file = OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .create(true)
                        .open(write_to)?;
                    if let Err(error) = file.write_all(multi_file_item.content.as_bytes()) {
                        error!("Error writing {}: {}", multi_file_item.internal_type, error);
                    }
                }
            }
            WriteTypesResult::SingleFile(single) => {
                let mut out = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(out.as_ref().join(language.get_config().default_file_name()))?;
                out.write_all(single.as_bytes())?;
                return Ok(());
            }
        };
    }
    let mut out = if out.as_ref().is_dir() {
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(out.as_ref().join(language.get_config().default_file_name()))
    } else {
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(out.as_ref())
    }?;
    let WriteTypesResult::SingleFile(file) = language.generate_from_parse(&data)? else {
        unreachable!("Multi file was already checked")
    };

    out.write_all(file.as_bytes())?;

    Ok(())
}

pub fn generate_parse<P: Parser, E: Error>(
    parser: &P,
    mut directories: VecDeque<String>,
) -> Result<ParsedData, ProcessInputError<P::Error, E>> {
    if directories.is_empty() {
        warn!("No directories specified. Exiting.");
        return Ok(ParsedData::default());
    }
    let mut types = TypesBuilder::new();
    for extension in P::file_extensions() {
        types
            .add(P::parser_name(), &format!("*.{}", extension))
            .unwrap();
    }
    types.select(P::parser_name());
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
        source.build_from_path(filepath.path());
        let data = std::fs::read_to_string(filepath.path())?;
        let result = parser.parse_from_str(&data, source);
        match result {
            Ok(ok) => {
                debug!("Parsed file {}", filepath.path().display());
                parsed_data = parsed_data + ok;
            }
            Err(err) => {
                error!(
                    "Error parsing file: {:?} file: {}",
                    err,
                    filepath.path().display()
                );
            }
        }
    }
    Ok(parsed_data)
}

pub fn process_directories_and_write<P: Parser, L: Language>(
    parser: &P,
    language: &mut L,
    directories: VecDeque<String>,
    out: impl AsRef<Path>,
) -> Result<(), ProcessInputError<P::Error, L::Error>> {
    let result = generate_parse(parser, directories)?;
    write_parse(language, result, out)?;
    Ok(())
}
pub fn process_input_and_write<P: Parser, L: Language>(
    parser: &P,
    language: &mut L,
    input: &str,
    out: impl AsRef<Path>,
    source: Source,
) -> Result<(), ProcessInputError<P::Error, L::Error>> {
    let data = parser
        .parse_from_str(input, source)
        .map_err(ProcessInputError::<P::Error, L::Error>::ParserError)?;
    write_parse(language, data, out)?;
    Ok(())
}
