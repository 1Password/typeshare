//! The core library for typeshare.
//! Contains the parser and language converters.
pub mod context;
pub mod error;
/// Implementations for each language converter
pub mod language;
/// Parsing Rust code into a format the `language` modules can understand
pub mod parser;
pub mod reconcile;
mod rename;
/// Codifying Rust types and how they convert to various languages.
pub mod rust_types;
mod target_os_check;
mod topsort;
mod visitors;

pub use rename::RenameExt;
