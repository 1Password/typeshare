/*!
typeshare-model is the core dependency of all typeshare implementations. It
defines the [`Language`] trait, which should be implemented by all aspiring
typeshare implementations, as well as a
 */
pub mod config;
pub mod decorator;
mod language;
pub mod parsed_data;

pub use language::*;

pub mod prelude {
    pub use crate::language::{FilesMode, Language};
    pub use crate::parsed_data::{
        CrateName, Id, ImportedType, RustConst, RustConstExpr, RustEnum, RustEnumShared,
        RustEnumVariant, RustEnumVariantShared, RustField, RustStruct, RustType, RustTypeAlias,
        SpecialRustType, TypeName,
    };
}
