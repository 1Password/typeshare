mod comment;
mod decorators;
mod enum_type;
mod generics;
mod other_types;
mod struct_type;
mod type_alias;
mod types;

#[doc(inline)]
pub use crate::parsed_types::comment::{Comment, CommentLocation};
#[doc(inline)]
pub use crate::parsed_types::decorators::{Decorator, Decorators, DecoratorsMap};
#[doc(inline)]
pub use crate::parsed_types::enum_type::{
    AlgebraicEnum, AnonymousStructVariant, EnumShared, EnumVariant, EnumVariantShared, ParsedEnum,
    TupleVariant,
};
#[doc(inline)]
pub use crate::parsed_types::generics::Generics;
#[doc(inline)]
pub use crate::parsed_types::other_types::{Id, Source};
#[doc(inline)]
pub use crate::parsed_types::struct_type::{ParsedStruct, StructShared};
#[doc(inline)]
pub use crate::parsed_types::type_alias::ParsedTypeAlias;
#[doc(inline)]
pub use crate::parsed_types::types::{Number, SpecialType, Type, TypeError};

use std::ops::Add;

/// The results of parsing Rust source input.
#[derive(Default, Debug)]
pub struct ParsedData {
    /// Structs defined in the source
    pub structs: Vec<ParsedStruct>,
    /// Enums defined in the source
    pub enums: Vec<ParsedEnum>,
    /// Type aliases defined in the source
    pub aliases: Vec<ParsedTypeAlias>,
}
impl Add for ParsedData {
    type Output = Self;

    fn add(mut self, other: Self) -> Self::Output {
        self.structs.extend(other.structs);
        self.enums.extend(other.enums);
        self.aliases.extend(other.aliases);
        self
    }
}

impl ParsedData {
    #[inline]
    pub fn add_struct(&mut self, parsed: ParsedStruct) {
        self.structs.push(parsed);
    }
    #[inline]
    pub fn add_enum(&mut self, parsed: ParsedEnum) {
        self.enums.push(parsed);
    }
    #[inline]
    pub fn add_type_alias(&mut self, parsed: ParsedTypeAlias) {
        self.aliases.push(parsed);
    }
    #[inline]
    pub fn push_item(&mut self, item: Item) {
        match item {
            Item::Struct(s) => self.structs.push(s),
            Item::Enum(e) => self.enums.push(e),
            Item::Alias(a) => self.aliases.push(a),
        }
    }
}

/// An enum that encapsulates units of code generation for Typeshare.
/// Analogous to `syn::Item`, even though our variants are more limited.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-everything",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde-everything", serde(tag = "type", content = "value"))]
pub enum Item {
    /// A `struct` definition
    Struct(ParsedStruct),
    /// An `enum` definition
    Enum(ParsedEnum),
    /// A `type` definition or newtype struct.
    Alias(ParsedTypeAlias),
}

/// Rust field definition.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-everything",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct Field {
    /// Identifier for the field.
    pub id: Id,
    /// Type of the field.
    pub ty: Type,
    /// Comments that were in the original source.
    pub comments: Comment,
    /// This will be true if the field has a `serde(default)` decorator.
    /// Even if the field's type is not optional, we need to make it optional
    /// for the languages we generate code for.
    pub has_default: bool,
    /// Language-specific decorators assigned to a given field.
    /// The keys are language names (e.g. SupportedLanguage::TypeScript), the values are field decorators (e.g. readonly)
    pub lang_decorators: DecoratorsMap,
}

impl Field {
    /// Returns an type override, if it exists, on this field for a given language.
    pub fn type_override(&self, language: impl AsRef<str>) -> Option<&str> {
        self.lang_decorators
            .get(language.as_ref())?
            .iter()
            .find_map(|fd| match fd {
                Decorator::LangType(ty) => Some(ty.as_str()),
                _ => None,
            })
    }
}
