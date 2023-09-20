use crate::parsed_types::comment::Comment;
use crate::parsed_types::{DecoratorsMap, Field, Generics, Id, Source, Type};
use std::ops::{Deref, DerefMut};
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-everything",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct StructShared {
    pub source: Source,
    /// The identifier for the struct.
    pub id: Id,
    /// The generic parameters that come after the struct name.
    pub generic_types: Generics,
    /// Comments that were in the struct source.
    /// We copy comments over to the typeshared files,
    /// so we need to collect them here.
    pub comments: Comment,
    /// Attributes that exist for this struct.
    pub decorators: DecoratorsMap,
}
/// Rust struct.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-everything",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde-everything", serde(tag = "type", content = "value"))]
pub enum ParsedStruct {
    TraditionalStruct {
        /// The fields of the struct.
        fields: Vec<Field>,
        /// Shared context for this struct.
        shared: StructShared,
    },
    SerializedAs {
        shared: StructShared,
        value_type: Type,
    },
}
impl ParsedStruct {
    pub fn shared(&self) -> StructShared {
        match self {
            Self::TraditionalStruct { shared, .. } => shared.clone(),
            Self::SerializedAs { shared, .. } => shared.clone(),
        }
    }
}
impl Deref for ParsedStruct {
    type Target = StructShared;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::TraditionalStruct { shared, .. } => shared,
            Self::SerializedAs { shared, .. } => shared,
        }
    }
}

impl DerefMut for ParsedStruct {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::TraditionalStruct { shared, .. } => shared,
            Self::SerializedAs { shared, .. } => shared,
        }
    }
}
