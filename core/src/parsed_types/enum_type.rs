use std::ops::{Deref, DerefMut};

use strum::EnumIs;

use crate::parsed_types::{Comment, DecoratorsMap, Field, Generics, Id, Source, Type};

/// Parsed information about a Rust enum definition
#[derive(Debug, Clone, PartialEq, EnumIs)]
#[cfg_attr(
    feature = "serde-everything",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde-everything", serde(tag = "type", content = "value"))]
pub enum ParsedEnum {
    SerializedAs {
        value_type: Type,
        shared: EnumShared,
    },
    /// A unit enum
    ///
    /// An example of such an enum:
    ///
    /// ```
    /// enum UnitEnum {
    ///     Variant,
    ///     AnotherVariant,
    ///     Yay,
    /// }
    /// ```
    Unit(EnumShared),
    /// An algebraic enum
    ///
    /// An example of such an enum:
    ///
    /// ```
    /// struct AssociatedData {/* ... */}
    ///
    /// enum AlgebraicEnum {
    ///     UnitVariant,
    ///     TupleVariant(AssociatedData),
    ///     AnonymousStruct { field: String, another_field: bool },
    /// }
    /// ```
    Algebraic(AlgebraicEnum),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-everything",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct AlgebraicEnum {
    /// The parsed value of the `#[serde(tag = "...")]` attribute
    pub tag_key: String,
    /// The parsed value of the `#[serde(content = "...")]` attribute
    pub content_key: String,
    /// Shared context for this enum.
    pub shared: EnumShared,
}
impl ParsedEnum {
    pub fn shared(&self) -> &EnumShared {
        match self {
            Self::Unit(shared)
            | Self::Algebraic(AlgebraicEnum { shared, .. })
            | Self::SerializedAs { shared, .. } => shared,
        }
    }
}
impl Deref for ParsedEnum {
    type Target = EnumShared;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Unit(shared)
            | Self::Algebraic(AlgebraicEnum { shared, .. })
            | Self::SerializedAs { shared, .. } => shared,
        }
    }
}
impl DerefMut for ParsedEnum {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Unit(shared)
            | Self::Algebraic(AlgebraicEnum { shared, .. })
            | Self::SerializedAs { shared, .. } => shared,
        }
    }
}

/// Enum information shared among different enum types
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-everything",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct EnumShared {
    pub source: Source,
    /// The enum's ident
    pub id: Id,
    /// Generic parameters for the enum, e.g. `SomeEnum<T>` would produce `vec!["T"]`
    pub generic_types: Generics,
    /// Comments on the enum definition itself
    pub comments: Comment,
    /// The enum's variants
    pub variants: Vec<EnumVariant>,
    /// Decorators applied to the enum for generation in other languages
    ///
    /// Example: `#[typeshare(swift = "Equatable, Comparable, Hashable")]`.
    pub lang_decorators: DecoratorsMap,
    /// True if this enum references itself in any field of any variant
    /// Swift needs the special keyword `indirect` for this case
    pub is_recursive: bool,
}

/// Parsed information about a Rust enum variant
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-everything",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde-everything", serde(tag = "type", content = "value"))]
pub enum EnumVariant {
    /// A unit variant
    Unit(EnumVariantShared),
    /// A tuple variant
    Tuple(TupleVariant),
    /// An anonymous struct variant
    AnonymousStruct(AnonymousStructVariant),
}
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-everything",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct TupleVariant {
    /// The type of the single tuple field
    pub ty: Type,
    /// Shared context for this enum.
    pub shared: EnumVariantShared,
}
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-everything",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct AnonymousStructVariant {
    /// The fields of the anonymous struct
    pub fields: Vec<Field>,
    /// Shared context for this enum.
    pub shared: EnumVariantShared,
}
impl EnumVariant {
    pub fn shared(&self) -> &EnumVariantShared {
        match self {
            Self::Unit(shared)
            | Self::Tuple(TupleVariant { shared, .. })
            | Self::AnonymousStruct(AnonymousStructVariant { shared, .. }) => shared,
        }
    }
}
impl Deref for EnumVariant {
    type Target = EnumVariantShared;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Unit(shared)
            | Self::Tuple(TupleVariant { shared, .. })
            | Self::AnonymousStruct(AnonymousStructVariant { shared, .. }) => shared,
        }
    }
}

impl DerefMut for EnumVariant {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Unit(shared)
            | Self::Tuple(TupleVariant { shared, .. })
            | Self::AnonymousStruct(AnonymousStructVariant { shared, .. }) => shared,
        }
    }
}

/// Variant information shared among different variant types
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-everything",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct EnumVariantShared {
    /// The variant's ident
    pub id: Id,
    /// Comments applied to the variant
    pub comments: Comment,
}
