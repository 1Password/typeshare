//! The core library for typeshare.
//! Contains the parser and language converters.

use language::Language;
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
    out: &mut dyn Write,
) -> Result<(), ProcessInputError> {
    let parsed_data = parser::parse(input)?;
    language.generate_types(out, &parsed_data)?;
    Ok(())
}
