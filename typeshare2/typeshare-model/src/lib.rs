pub mod decorator;
pub mod language;
pub mod parsed_data;

pub use language::Language;

pub mod prelude {
    pub use crate::language::{CrateTypes, FilesMode, Language, ScopedCrateTypes};
    pub use crate::parsed_data::{
        CrateName, Id, ImportedType, ParsedData, RustEnum, RustEnumShared, RustEnumVariant,
        RustEnumVariantShared, RustField, RustItem, RustStruct, RustType, RustTypeAlias,
        SpecialRustType, TypeName,
    };
}
