//! The core library for typeshare.
//! Contains the parser and language converters.

use language::{CrateTypes, Language};
use std::io::Write;
use thiserror::Error;

mod rename;

/// Implementations for each language converter
pub mod language;
/// Parsing Rust code into a format the `language` modules can understand
pub mod parser;
/// Codifying Rust types and how they convert to various languages.
pub mod rust_types;
mod topsort;
mod visitors;

#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum ProcessInputError {
    #[error("a parsing error occurred: {0}")]
    ParseError(#[from] parser::ParseError),
    #[error("a type generation error occurred: {0}")]
    IoError(#[from] std::io::Error),
}

/// Parse and generate types for a single Rust input file.
pub fn process_input(
    input: &str,
    language: &mut dyn Language,
    imports: &CrateTypes,
    out: &mut dyn Write,
) -> Result<(), ProcessInputError> {
    let mut parsed_data = parser::parse(
        input,
        "default_name".into(),
        "file_name".into(),
        "file_path".into(),
        &[],
    )?
    .unwrap();

    if !parsed_data.errors.is_empty() {
        return Err(ProcessInputError::ParseError(
            parsed_data.errors.remove(0).error,
        ));
    }

    language.generate_types(out, imports, parsed_data)?;
    Ok(())
}
