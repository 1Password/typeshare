//! The core library for typeshare.
//! Contains the parser and language converters.

use crate::language::LanguageError;
use std::error::Error;
use std::io;
use thiserror::Error;

pub mod rename;

pub mod config;
/// Implementations for each language converter
pub mod language;
pub mod parsed_types;
pub mod parser;

/// Parsing Rust code into a format the `language` modules can understand
pub mod topsort;
mod utils;
pub mod value;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub use utils::*;
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum ProcessInputError<P: Error, L: Error> {
    #[error(transparent)]
    ParserError(P),
    #[error(transparent)]
    LanguageError(#[from] LanguageError<L>),
    #[error("An error occurred while reading the input: {0}")]
    IgnoreError(#[from] ignore::Error),
    #[error("An error occurred while reading the input: {0}")]
    IO(#[from] io::Error),
}
#[test]
pub fn test() {
    println!("Hello, world!");
}
