//! The core library for typeshare.
//! Contains the parser and language converters.

use std::ffi::c_int;
use std::{
    error::Error,
    fs::{create_dir_all, OpenOptions},
    io,
    io::Write,
    path::Path,
};

use language::Language;
use log::{error, info};
#[cfg(feature = "rust-parsing")]
pub use rust_parser::pub_utils::*;
use thiserror::Error;
pub use topsort::topsort;

use crate::{
    language::{Generate, LanguageConfig, LanguageError, WriteTypesResult},
    parsed_types::{Comment, CommentLocation, ParsedData},
};

mod rename;

#[cfg(feature = "cli")]
pub mod cli;
/// Implementations for each language converter
pub mod language;
mod language_desc;
/// Codifying Rust types and how they convert to various languages.
pub mod parsed_types;
/// Parsing Rust code into a format the `language` modules can understand
#[cfg(feature = "rust-parsing")]
pub mod rust_parser;
mod topsort;
use crate::rust_parser::ParseError;
#[doc(inline)]
pub use language_desc::{FFILanguageDescription, LanguageDescription};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
#[no_mangle]
pub static TYPESHARE_FFI_VERSION: c_int = 1;

#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum ProcessInputError<E: Error> {
    /// Parsing Rust code into a format the `language` modules can understand
    #[cfg(feature = "rust-parsing")]
    #[error("a parsing error occurred: {0}")]
    ParseError(#[from] ParseError),
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
pub fn write_into_one<L: Language>(
    language: &mut L,
    data: ParsedData,
    out: &mut impl Write,
) -> Result<(), ProcessInputError<L::Error>>
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
