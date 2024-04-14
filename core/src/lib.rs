//! The core library for typeshare.
//! Contains the parser and language converters.

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
