use std::path::PathBuf;

use crate::language::CrateName;

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
    pub source_code: String,
    pub crate_name: CrateName,
    pub file_name: String,
    pub file_path: PathBuf,
}
