use log::debug;
use quote::ToTokens;
use std::collections::BTreeSet;
use std::fmt::Display;
use std::str::FromStr;
use std::{collections::HashMap, convert::TryFrom};
use syn::spanned::Spanned;
use syn::{Expr, ExprLit, Lit, TypeArray, TypeSlice};
use thiserror::Error;

use crate::error::{ParseErrorWithSpan, RustTypeParseError, WithSpan as _};
use crate::language::SupportedLanguage;
use crate::parser::DecoratorKind;
use crate::visitors::accept_type;

/// Type level typeshare attributes are mapped by target language and a mapping of attribute.
pub type DecoratorMap = HashMap<DecoratorKind, BTreeSet<String>>;

/// Identifier used in Rust structs, enums, and fields. It includes the `original` name and the `renamed` value after the transformation based on `serde` attributes.
#[derive(Debug, Clone, PartialEq)]
pub struct Id {
    /// The original identifier name
    pub original: String,
    /// The renamed identifier, based on serde attributes.
    /// If there is no re-naming going on, this will be identical to
    /// `original`.
    pub renamed: String,
    /// Was this renamed with `serde(rename = "newname")
    pub serde_rename: bool,
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.original == self.renamed {
            write!(f, "({})", self.original)
        } else {
            write!(f, "({}, {})", self.original, self.renamed)
        }
    }
}

/// Rust struct.
#[derive(Debug, Clone)]
pub struct RustStruct {
    /// The identifier for the struct.
    pub id: Id,
    /// The generic parameters that come after the struct name.
    pub generic_types: Vec<String>,
    /// The fields of the struct.
    pub fields: Vec<RustField>,
    /// Comments that were in the struct source.
    /// We copy comments over to the typeshared files,
    /// so we need to collect them here.
    pub comments: Vec<String>,
    /// Attributes that exist for this struct.
    pub decorators: DecoratorMap,
    /// True if this struct contains data that needs to be redacted
    pub is_redacted: bool,
}

impl PartialEq for RustStruct {
    fn eq(&self, other: &Self) -> bool {
        self.id.original == other.id.original
    }
}

impl Eq for RustStruct {}

impl PartialOrd for RustStruct {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RustStruct {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.original.cmp(&other.id.original)
    }
}

/// Rust const variable.
///
/// Typeshare can only handle numeric and string constants.
/// ```
/// pub const MY_CONST: &str = "constant value";
/// ```
#[derive(Debug, Clone)]
pub struct RustConst {
    /// The identifier for the constant.
    pub id: Id,
    /// The type identifier that this constant is referring to.
    pub r#type: RustType,
    /// The expression that the constant contains.
    pub expr: RustConstExpr,
}

impl PartialEq for RustConst {
    fn eq(&self, other: &Self) -> bool {
        self.id.original == other.id.original
    }
}

/// A constant expression that can be shared via a constant variable across the typeshare
/// boundary.
#[derive(Debug, Clone)]
pub enum RustConstExpr {
    /// Expression represents an integer.
    Int(i128),
}

/// Rust type alias.
/// ```
/// pub struct MasterPassword(String);
/// ```
#[derive(Debug, Clone)]
pub struct RustTypeAlias {
    /// The identifier for the alias.
    pub id: Id,
    /// The generic parameters that come after the type alias name.
    pub generic_types: Vec<String>,
    /// The type identifier that this type alias is aliasing
    pub r#type: RustType,
    /// Comments that were in the type alias source.
    pub comments: Vec<String>,
    /// Attributes that exist for this struct.
    pub decorators: DecoratorMap,
    /// True if this type alias contains data that needs to be redacted
    pub is_redacted: bool,
}

impl PartialEq for RustTypeAlias {
    fn eq(&self, other: &Self) -> bool {
        self.id.original == other.id.original
    }
}

impl Eq for RustTypeAlias {}

impl Ord for RustTypeAlias {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.original.cmp(&other.id.original)
    }
}

impl PartialOrd for RustTypeAlias {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Rust field definition.
#[derive(Debug, Clone, PartialEq)]
pub struct RustField {
    /// Identifier for the field.
    pub id: Id,
    /// Type of the field.
    pub ty: RustType,
    /// Comments that were in the original source.
    pub comments: Vec<String>,
    /// This will be true if the field has a `serde(default)` decorator.
    /// Even if the field's type is not optional, we need to make it optional
    /// for the languages we generate code for.
    pub has_default: bool,
    /// Language-specific decorators assigned to a given field.
    /// The keys are language names (e.g. SupportedLanguage::TypeScript), the values are field decorators (e.g. readonly)
    pub decorators: HashMap<SupportedLanguage, BTreeSet<FieldDecorator>>,
}

/// A single decorator on a field in Rust code.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FieldDecorator {
    /// A boolean flag enabled by its existence as a decorator: for example, `readonly`.
    Word(String),
    /// A key-value pair, such as `type = "any"`.
    NameValue(String, String),
}

impl FieldDecorator {
    /// Returns the name of the field decorator. For a word decorator, this is just the identifier.
    pub fn name(&self) -> &str {
        match self {
            Self::Word(name) | Self::NameValue(name, _) => name,
        }
    }
}

/// A Rust type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustType {
    /// A type with generic parameters. Consists of a type ID + parameters that come
    /// after in angled brackets. Examples include:
    /// - `SomeStruct<String>`
    /// - `SomeEnum<u32>`
    /// - `SomeTypeAlias<(), &str>`
    ///   However, there are some generic types that are considered to be _special_. These
    ///   include `Vec<T>` `HashMap<K, V>`, and `Option<T>`, which are part of `SpecialRustType` instead
    ///   of `RustType::Generic`.
    ///
    /// If a generic type is type-mapped via `typeshare.toml`, the generic parameters will be dropped automatically.
    Generic {
        #[allow(missing_docs)]
        id: String,
        #[allow(missing_docs)]
        parameters: Vec<RustType>,
    },
    /// A type that requires a special transformation to its respective language. This includes
    /// many core types, like string types, basic container types, numbers, and other primitives.
    Special(SpecialRustType),
    /// A type with no generic parameters that is not considered a **special** type. This includes
    /// all user-generated types and some types from the standard library or third-party crates.
    /// However, these types can still be transformed as part of the type-map in `typeshare.toml`.
    Simple {
        #[allow(missing_docs)]
        id: String,
    },
}

impl Display for RustType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rust_type = match &self {
            RustType::Simple { id } => id,
            RustType::Generic { id, parameters } => {
                if parameters.is_empty() {
                    id
                } else {
                    &format!(
                        "{id}<{}>",
                        parameters
                            .iter()
                            .map(|p| p.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            RustType::Special(ty) => &ty.to_string(),
        };
        write!(f, "{rust_type}")
    }
}

/// A special rust type that needs a manual type conversion
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecialRustType {
    /// Represents `Vec<T>` from the standard library
    Vec(Box<RustType>),
    /// Represents `[T; N]` from the standard library
    Array(Box<RustType>, usize),
    /// Represents `&[T]` from the standard library
    Slice(Box<RustType>),
    /// Represents `HashMap<K, V>` from the standard library
    HashMap(Box<RustType>, Box<RustType>),
    /// Represents `Option<T>` from the standard library
    Option(Box<RustType>),
    /// Represents time::OffsetDateTime from time
    /// We serialize/deserialize this to an UTC time specifically
    /// encoded in the RFC3339 or ISO8601 format.
    /// This should be used with serde's with tag when serializing/deserializing
    /// like so #[serde(with = "time::serde::rfc3339")]
    DateTime,
    /// Represents `()`
    Unit,
    /// Represents `String` from the standard library
    String,
    /// Represents `char`
    Char,
    /// Represents `i8`
    I8,
    /// Represents `i16`
    I16,
    /// Represents `i32`
    I32,
    /// Represents `i64`
    I64,
    /// Represents `u128`
    U128,
    /// Represents `u8`
    U8,
    /// Represents `u16`
    U16,
    /// Represents `u32`
    U32,
    /// Represents `u64`
    U64,
    /// Represents `isize`
    ISize,
    /// Represents `usize`
    USize,
    /// Represents `bool`
    Bool,
    /// Represents `f32`
    F32,
    /// Represents `f64`
    F64,
    /// Represents `I54` from `typeshare::I54`
    I54,
    /// Represents `U53` from `typeshare::U53`
    U53,
}

impl Display for SpecialRustType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let special_type = match self {
            SpecialRustType::Vec(rust_type) => format!("Vec<{rust_type}>"),
            SpecialRustType::Array(rust_type, _) => format!("[{rust_type}]"),
            SpecialRustType::Slice(rust_type) => format!("&[{rust_type}]"),
            SpecialRustType::HashMap(rust_type, rust_type1) => {
                format!("HashMap<{rust_type},{rust_type1}>")
            }
            SpecialRustType::Option(rust_type) => {
                format!("Option<{}>", rust_type.id())
            }
            _ => self.id().to_string(),
        };
        write!(f, "{special_type}")
    }
}

impl FromStr for RustType {
    type Err = ParseErrorWithSpan;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let syn_type = syn::parse_str(s)
            .map_err(|err| RustTypeParseError::UnsupportedType(Vec::new()).with_span(err.span()))?;
        Self::try_from(&syn_type)
    }
}

impl TryFrom<&syn::Type> for RustType {
    type Error = ParseErrorWithSpan;

    fn try_from(ty: &syn::Type) -> Result<Self, Self::Error> {
        debug!("Parsing type: {}", ty.to_token_stream());
        Ok(match ty {
            syn::Type::Tuple(tuple) if tuple.elems.iter().count() == 0 => {
                Self::Special(SpecialRustType::Unit)
            }
            syn::Type::Tuple(tt) => {
                return Err(RustTypeParseError::UnexpectedParameterizedTuple.with_span(tt.span()))
            }
            syn::Type::Reference(reference) => Self::try_from(reference.elem.as_ref())?,
            syn::Type::Path(path) => {
                let segment = path.path.segments.iter().next_back().unwrap();
                let id = segment.ident.to_string();
                let parameters: Vec<Self> = match &segment.arguments {
                    syn::PathArguments::AngleBracketed(angle_bracketed_arguments) => {
                        let parameters: Result<Vec<Self>, Self::Error> = angle_bracketed_arguments
                            .args
                            .iter()
                            .filter_map(|arg| match arg {
                                syn::GenericArgument::Type(r#type) => Some(Self::try_from(r#type)),
                                _ => None,
                            })
                            .collect();
                        parameters?
                    }
                    _ => Vec::default(),
                };
                match id.as_str() {
                    "Vec" => Self::Special(SpecialRustType::Vec(
                        parameters.into_iter().next().unwrap().into(),
                    )),
                    "Option" => Self::Special(SpecialRustType::Option(
                        parameters.into_iter().next().unwrap().into(),
                    )),
                    "HashMap" => {
                        let mut params = parameters.into_iter();
                        Self::Special(SpecialRustType::HashMap(
                            params.next().unwrap().into(),
                            params.next().unwrap().into(),
                        ))
                    }
                    "OffsetDateTime" => Self::Special(SpecialRustType::DateTime),
                    "str" | "String" => Self::Special(SpecialRustType::String),
                    // These smart pointers can be treated as their inner type since serde can handle it
                    // See impls of serde::Deserialize
                    "Box" | "Weak" | "Arc" | "Rc" | "Cow" | "ArcWeak" | "RcWeak" | "Cell"
                    | "Mutex" | "RefCell" | "RwLock" => parameters.into_iter().next().unwrap(),
                    "bool" => Self::Special(SpecialRustType::Bool),
                    "char" => Self::Special(SpecialRustType::Char),
                    "u8" => Self::Special(SpecialRustType::U8),
                    "u16" => Self::Special(SpecialRustType::U16),
                    "u32" => Self::Special(SpecialRustType::U32),
                    "U53" => Self::Special(SpecialRustType::U53),
                    "u64" => Self::Special(SpecialRustType::U64),
                    "usize" => Self::Special(SpecialRustType::USize),
                    "u128" => Self::Special(SpecialRustType::U128),
                    "i8" => Self::Special(SpecialRustType::I8),
                    "i16" => Self::Special(SpecialRustType::I16),
                    "i32" => Self::Special(SpecialRustType::I32),
                    "i64" => Self::Special(SpecialRustType::I64),
                    "isize" => Self::Special(SpecialRustType::ISize),
                    "I54" => Self::Special(SpecialRustType::I54),
                    "f32" => Self::Special(SpecialRustType::F32),
                    "f64" => Self::Special(SpecialRustType::F64),
                    _ => {
                        if parameters.is_empty() {
                            Self::Simple { id }
                        } else {
                            Self::Generic { id, parameters }
                        }
                    }
                }
            }
            syn::Type::Array(TypeArray {
                elem,
                len:
                    Expr::Lit(ExprLit {
                        lit: Lit::Int(count),
                        ..
                    }),
                ..
            }) => Self::Special(SpecialRustType::Array(
                Self::try_from(elem.as_ref())?.into(),
                count.base10_parse().map_err(|err| {
                    let span = err.span();
                    RustTypeParseError::NumericLiteral(err).with_span(span)
                })?,
            )),
            syn::Type::Slice(TypeSlice {
                bracket_token: _,
                elem,
            }) => Self::Special(SpecialRustType::Slice(
                Self::try_from(elem.as_ref())?.into(),
            )),
            ty => {
                return Err(
                    RustTypeParseError::UnexpectedToken(ty.to_token_stream().to_string())
                        .with_span(ty.span()),
                )
            }
        })
    }
}

impl RustType {
    /// Check if a type contains a type with an ID that matches `ty`.
    /// For example, `Box<String>` contains the types `Box` and `String`. Similarly,
    /// `Vec<Option<HashMap<String, Url>>>` contains the types `Vec`, `Option`, `HashMap`,
    /// `String`, and `Url`.
    pub fn contains_type(&self, ty: &str) -> bool {
        match &self {
            Self::Simple { id } => id == ty,
            Self::Generic { id, parameters } => {
                id == ty || parameters.iter().any(|p| p.contains_type(ty))
            }
            Self::Special(special) => special.contains_type(ty),
        }
    }

    /// Get the ID (AKA name) of the type.
    pub fn id(&self) -> &str {
        match &self {
            Self::Simple { id } | Self::Generic { id, .. } => id.as_str(),
            Self::Special(special) => special.id(),
        }
    }
    /// Check if the type is `Option<T>`
    pub fn is_optional(&self) -> bool {
        matches!(self, Self::Special(SpecialRustType::Option(_)))
    }

    /// Check if the type is `Option<Option<T>>`
    pub fn is_double_optional(&self) -> bool {
        match &self {
            RustType::Special(SpecialRustType::Option(t)) => {
                matches!(t.as_ref(), RustType::Special(SpecialRustType::Option(_)))
            }
            _ => false,
        }
    }
    /// Check if the type is `Vec<T>`
    pub fn is_vec(&self) -> bool {
        matches!(self, Self::Special(SpecialRustType::Vec(_)))
    }
    /// Check if the type is `HashMap<K, V>`
    pub fn is_hash_map(&self) -> bool {
        matches!(self, Self::Special(SpecialRustType::HashMap(_, _)))
    }
    /// Get the generic parameters for this type. Returns an empty iterator if there are none.
    /// For example, `Vec<String>`'s generic parameters would be `[String]`.
    /// Meanwhile, `HashMap<i64, u32>`'s generic parameters would be `[i64, u32]`.
    /// Finally, a type like `String` would have no generic parameters.
    pub fn parameters(&self) -> Box<dyn Iterator<Item = &Self> + '_> {
        match &self {
            Self::Simple { .. } => Box::new(std::iter::empty()),
            Self::Generic { parameters, .. } => Box::new(parameters.iter()),
            Self::Special(special) => special.parameters(),
        }
    }

    /// Yield all the type names including nested generic types.
    pub fn all_reference_type_names(&self) -> impl Iterator<Item = &'_ str> + '_ {
        RustRefTypeIter {
            ty: Some(self),
            parameters: Vec::new(),
        }
        .filter(|s| accept_type(s))
    }
}

struct RustRefTypeIter<'a> {
    ty: Option<&'a RustType>,
    parameters: Vec<&'a RustType>,
}

impl<'a> Iterator for RustRefTypeIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(t) = self.parameters.pop() {
            self.parameters.extend(t.parameters());
            return Some(t.id());
        }

        if let Some(t) = self.ty.take() {
            self.parameters = t.parameters().collect();
            return Some(t.id());
        }

        None
    }
}

impl RustField {
    /// Returns an type override, if it exists, on this field for a given language.
    pub fn type_override(&self, language: SupportedLanguage) -> Option<&str> {
        self.decorators
            .get(&language)?
            .iter()
            .find_map(|fd| match fd {
                FieldDecorator::NameValue(name, ty) if name == "type" => Some(ty.as_str()),
                _ => None,
            })
    }
}

#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum RustTypeFormatError {
    #[error("Generic parameter `{0}` is forbidden in Go")]
    GenericsForbiddenInGo(String),
    #[error("Generic type `{0}` cannot be used as a map key in Typescript")]
    GenericKeyForbiddenInTS(String),
    #[error("The special type `{0}` is not supported in this language")]
    UnsupportedSpecialType(String),
}

impl SpecialRustType {
    /// Check if this type is equivalent to or contains `ty` in one of its generic parameters.
    pub fn contains_type(&self, ty: &str) -> bool {
        match &self {
            Self::Vec(rty) | Self::Array(rty, _) | Self::Slice(rty) | Self::Option(rty) => {
                rty.contains_type(ty)
            }
            Self::HashMap(rty1, rty2) => rty1.contains_type(ty) || rty2.contains_type(ty),
            Self::Unit
            | Self::String
            | Self::DateTime
            | Self::Char
            | Self::I8
            | Self::I16
            | Self::I32
            | Self::I64
            | Self::U8
            | Self::U16
            | Self::U32
            | Self::U64
            | Self::ISize
            | Self::USize
            | Self::Bool
            | Self::F32
            | Self::F64
            | Self::I54
            | Self::U128
            | Self::U53 => ty == self.id(),
        }
    }

    /// Returns the Rust identifier for this special type.
    pub fn id(&self) -> &'static str {
        match &self {
            Self::Unit => "()",
            Self::F64 => "f64",
            Self::F32 => "f32",
            Self::Vec(_) => "Vec",
            Self::Array(_, _) => "[]",
            Self::Slice(_) => "&[]",
            Self::Option(_) => "Option",
            Self::HashMap(_, _) => "HashMap",
            Self::DateTime => "OffsetDateTime",
            Self::String => "String",
            Self::Char => "char",
            Self::Bool => "bool",
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::ISize => "isize",
            Self::USize => "usize",
            Self::U53 => "U53",
            Self::I54 => "I54",
            Self::U128 => "u128",
        }
    }

    /// Iterate over the generic parameters for this type. Returns an empty iterator
    /// if there are none.
    pub fn parameters(&self) -> Box<dyn Iterator<Item = &RustType> + '_> {
        match &self {
            Self::Vec(rtype) | Self::Array(rtype, _) | Self::Slice(rtype) | Self::Option(rtype) => {
                Box::new(std::iter::once(rtype.as_ref()))
            }
            Self::HashMap(rtype1, rtype2) => {
                Box::new([rtype1.as_ref(), rtype2.as_ref()].into_iter())
            }
            Self::Unit
            | Self::String
            | Self::DateTime
            | Self::Char
            | Self::I8
            | Self::I16
            | Self::I32
            | Self::I64
            | Self::U8
            | Self::U16
            | Self::U32
            | Self::U64
            | Self::U128
            | Self::ISize
            | Self::USize
            | Self::Bool
            | Self::F32
            | Self::F64
            | Self::I54
            | Self::U53 => Box::new(std::iter::empty()),
        }
    }
}

/// Parsed information about a Rust enum definition
#[derive(Debug, Clone)]
pub enum RustEnum {
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
    Unit(RustEnumShared),
    /// An algebraic enum
    ///
    /// An example of such an enum:
    ///
    /// ```
    /// struct AssociatedData { /* ... */ }
    ///
    /// enum AlgebraicEnum {
    ///     UnitVariant,
    ///     TupleVariant(AssociatedData),
    ///     AnonymousStruct {
    ///         field: String,
    ///         another_field: bool,
    ///     },
    /// }
    /// ```
    Algebraic {
        /// The parsed value of the `#[serde(tag = "...")]` attribute
        tag_key: String,
        /// The parsed value of the `#[serde(content = "...")]` attribute
        content_key: String,
        /// Shared context for this enum.
        shared: RustEnumShared,
    },
}

impl PartialEq for RustEnum {
    fn eq(&self, other: &Self) -> bool {
        self.shared().id.original == other.shared().id.original
    }
}

impl Eq for RustEnum {}

impl PartialOrd for RustEnum {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RustEnum {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.shared().id.original.cmp(&other.shared().id.original)
    }
}

impl RustEnum {
    /// Get a reference to the inner shared content
    pub fn shared(&self) -> &RustEnumShared {
        match self {
            Self::Unit(shared) | Self::Algebraic { shared, .. } => shared,
        }
    }
}

/// Enum information shared among different enum types
#[derive(Debug, Clone, PartialEq)]
pub struct RustEnumShared {
    /// The enum's ident
    pub id: Id,
    /// Generic parameters for the enum, e.g. `SomeEnum<T>` would produce `vec!["T"]`
    pub generic_types: Vec<String>,
    /// Comments on the enum definition itself
    pub comments: Vec<String>,
    /// The enum's variants
    pub variants: Vec<RustEnumVariant>,
    /// Decorators applied to the enum for generation in other languages
    ///
    /// Example: `#[typeshare(swift = "Equatable, Comparable, Hashable")]`.
    pub decorators: DecoratorMap,
    /// True if this enum references itself in any field of any variant
    /// Swift needs the special keyword `indirect` for this case
    pub is_recursive: bool,
    /// True if this enum contains data that needs to be redacted
    pub is_redacted: bool,
}

/// Parsed information about a Rust enum variant
#[derive(Debug, Clone, PartialEq)]
pub enum RustEnumVariant {
    /// A unit variant
    Unit(RustEnumVariantShared),
    /// A tuple variant
    Tuple {
        /// The type of the single tuple field
        ty: RustType,
        /// Shared context for this enum.
        shared: RustEnumVariantShared,
    },
    /// An anonymous struct variant
    AnonymousStruct {
        /// The fields of the anonymous struct
        fields: Vec<RustField>,
        /// Shared context for this enum.
        shared: RustEnumVariantShared,
    },
}

impl RustEnumVariant {
    /// Get a reference to the inner shared content
    pub fn shared(&self) -> &RustEnumVariantShared {
        match self {
            Self::Unit(shared)
            | Self::Tuple { shared, .. }
            | Self::AnonymousStruct { shared, .. } => shared,
        }
    }
}

/// Variant information shared among different variant types
#[derive(Debug, Clone, PartialEq)]
pub struct RustEnumVariantShared {
    /// The variant's ident
    pub id: Id,
    /// Comments applied to the variant
    pub comments: Vec<String>,
}

/// An enum that encapsulates units of code generation for Typeshare.
/// Analogous to `syn::Item`, even though our variants are more limited.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum RustItem {
    /// A `struct` definition
    Struct(RustStruct),
    /// An `enum` definition
    Enum(RustEnum),
    /// A `type` definition or newtype struct.
    Alias(RustTypeAlias),
    /// A `const` definition
    Const(RustConst),
}
