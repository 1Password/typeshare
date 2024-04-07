//! The core library for typeshare.
//! Contains the parser and language converters.

use language::Language;
use std::{
    collections::{HashMap, HashSet},
    io::Write,
};
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
    imports: &HashMap<String, HashSet<String>>,
    out: &mut dyn Write,
) -> Result<(), ProcessInputError> {
    let parsed_data = parser::parse(
        input,
        "default_name".into(),
        "file_name".into(),
        "file_path",
    )?;
    language.generate_types(out, imports, &parsed_data.unwrap())?;
    Ok(())
}
