//! Context types for parsing.
//!
use crate::language::CrateName;
use std::path::PathBuf;

/// Context for parsing rust source files.
#[derive(Default)]
pub struct ParseContext<'a> {
    /// Types to ignore
    pub ignored_types: Vec<&'a str>,
    /// Multi file output enabled.
    pub multi_file: bool,
    /// `target_os` filtering.
    pub target_os: Vec<String>,
}

/// Parsing context for a single rust source file.
pub struct ParseFileContext {
    /// Source code content
    pub source_code: String,
    /// Name of the crate this file belongs to.
    pub crate_name: CrateName,
    /// File name.
    pub file_name: String,
    /// Full path to the source file.
    pub file_path: PathBuf,
}
