//! The core library for typeshare.
//! Contains the parser and language converters.
use thiserror::Error;

pub mod context;
/// Implementations for each language converter
pub mod language;
/// Parsing Rust code into a format the `language` modules can understand
pub mod parser;
mod rename;
/// Codifying Rust types and how they convert to various languages.
pub mod rust_types;
mod target_os_check;
mod topsort;
mod visitors;

pub use rename::RenameExt;

#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum ProcessInputError {
    #[error("a parsing error occurred: {0}")]
    ParseError(#[from] parser::ParseError),
    #[error("a type generation error occurred: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Error)]
/// Errors during file generation.
pub enum GenerationError {
    /// The post generation step failed.
    #[error("Post generation failed: {0}")]
    PostGeneration(String),
}
