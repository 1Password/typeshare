//! The core library for typeshare.
//! Contains the parser and language converters.

use crate::language::{Comment, CommentLocation, LanguageConfig, LanguageError, WriteTypesResult};
use crate::parser::ParsedData;
use crate::rust_types::Source;
use ignore::overrides::OverrideBuilder;
use ignore::types::TypesBuilder;
use ignore::WalkBuilder;
use language::Language;
use log::{debug, error, info, warn};
use std::collections::VecDeque;
use std::error::Error;
use std::fs::{create_dir_all, OpenOptions};
use std::io;
use std::io::Write;
use std::path::Path;
use thiserror::Error;

mod rename;

#[cfg(feature = "cli")]
pub mod cli;
/// Implementations for each language converter
pub mod language;
/// Parsing Rust code into a format the `language` modules can understand
pub mod parser;
/// Codifying Rust types and how they convert to various languages.
pub mod rust_types;
mod topsort;

#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum ProcessInputError<E: Error> {
    #[error("a parsing error occurred: {0}")]
    ParseError(#[from] parser::ParseError),
    #[error(transparent)]
    LanguageError(#[from] LanguageError<E>),
    #[error("Multiple files were generated, but only one was expected")]
    MultipleFilesGenerated,
    #[error("An error occurred while reading the input: {0}")]
    IgnoreError(#[from] ignore::Error),
}
impl<E: Error> From<io::Error> for ProcessInputError<E> {
    fn from(error: io::Error) -> Self {
        ProcessInputError::LanguageError(LanguageError::IoError(error))
    }
}

/// Will process the input and write the output to the given writer.
///
/// If multiple files are generated, they will be written to the given writer
/// With Comments indicating the beginning and end of each file.
pub fn process_into_one<L: Language>(
    language: &mut L,
    data: &str,
    out: &mut impl Write,
    source: Source,
) -> Result<(), ProcessInputError<L::Error>> {
    let data = parser::parse(data, source)?;
    match language.generate_types(&data)? {
        WriteTypesResult::MultiFile { files } => {
            for multi_file_item in files {
                language.write_comment(
                    out,
                    &Comment::new_single(
                        format!("Start of File {}", multi_file_item.name),
                        CommentLocation::FileHeader,
                    ),
                )?;
                out.write_all(multi_file_item.content.as_bytes())?;
                language.write_comment(
                    out,
                    &Comment::new_single(
                        format!("End of File {}", multi_file_item.name),
                        CommentLocation::FileHeader,
                    ),
                )?;
            }
        }
        WriteTypesResult::SingleFile(single) => {
            out.write_all(single.as_bytes())?;
        }
    }
    Ok(())
}
pub fn process_directory_and_write<L: Language>(
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
    for root in directories {
        walker_builder.add(root);
    }

    // Sort walker output for deterministic output across platforms
    walker_builder.sort_by_file_path(|a, b| a.cmp(b));
    walker_builder.types(types);
    walker_builder.overrides(overrides);

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
        let result = parser::parse(&data, source);
        match result {
            Ok(ok) => {
                debug!("Parsed file {}", filepath.path().display());
                parsed_data = parsed_data + ok;
            }
            Err(err) => {
                error!("Error parsing file: {}", err);
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
    let data = parser::parse(input, source)?;
    write_parse(language, data, out)?;
    Ok(())
}

pub fn write_parse<L: Language>(
    language: &mut L,
    data: ParsedData,
    out: impl AsRef<Path>,
) -> Result<(), LanguageError<L::Error>> {
    if language.multi_file() {
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

        match language.generate_types(&data)? {
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
    let WriteTypesResult::SingleFile(file) = language.generate_types(&data)? else {
        unreachable!("Multi file was already checked")
    };

    out.write_all(file.as_bytes())?;

    Ok(())
}
