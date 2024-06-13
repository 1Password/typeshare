pub mod rust_types;

use crate::rust_types::{RustEnum, RustStruct, RustTypeAlias};

/// The results of parsing Rust source input.
#[derive(Default, Debug)]
pub struct ParsedData {
    /// Structs defined in the source
    pub structs: Vec<RustStruct>,
    /// Enums defined in the source
    pub enums: Vec<RustEnum>,
    /// Type aliases defined in the source
    pub aliases: Vec<RustTypeAlias>,
    /// Imports used by this file
    pub import_types: HashSet<ImportedType>,
    /// Crate this belongs to.
    pub crate_name: CrateName,
    /// File name to write to for generated type.
    pub file_name: String,
    /// All type names
    pub type_names: HashSet<String>,
    /// Failures during parsing.
    pub errors: Vec<ErrorInfo>,
    /// Using multi file support.
    pub multi_file: bool,
}
