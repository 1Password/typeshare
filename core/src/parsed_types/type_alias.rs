use crate::parsed_types::{comment::Comment, Generics, Id, Source, Type};

/// Rust type alias.
/// ```
/// pub struct MasterPassword(String);
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-everything",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ParsedTypeAlias {
    pub source: Source,
    /// The identifier for the alias.
    pub id: Id,
    /// The generic parameters that come after the type alias name.
    pub generic_types: Generics,
    /// The type identifier that this type alias is aliasing
    pub value_type: Type,
    /// Comments that were in the type alias source.
    pub comments: Comment,
}
