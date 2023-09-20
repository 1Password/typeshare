use thiserror::Error;

#[derive(Debug, Error)]
pub enum TypeError {
    #[error("{0:?}")]
    UnsupportedType(Vec<String>),
    #[error("Unexpected token when parsing type: `{0}`. This is an internal error, please ping a typeshare developer to resolve this problem.")]
    UnexpectedToken(String),
    #[error("Tuples are not allowed in typeshare types")]
    UnexpectedParameterizedTuple,
    #[error("Could not parse numeric literal: {0:?}")]
    NumericLiteral(anyhow::Error),
}
/// A Rust type.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-everything",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde-everything", serde(tag = "type", content = "value"))]
pub enum Type {
    /// A type with generic parameters. Consists of a type ID + parameters that come
    /// after in angled brackets. Examples include:
    /// - `SomeStruct<String>`
    /// - `SomeEnum<u32>`
    /// - `SomeTypeAlias<(), &str>`
    /// However, there are some generic types that are considered to be _special_. These
    /// include `Vec<T>` `HashMap<K, V>`, and `Option<T>`, which are part of `SpecialRustType` instead
    /// of `RustType::Generic`.
    ///
    /// If a generic type is type-mapped via `typeshare.toml`, the generic parameters will be dropped automatically.
    Generic {
        #[allow(missing_docs)]
        id: String,
        #[allow(missing_docs)]
        parameters: Vec<Type>,
    },
    /// A type that requires a special transformation to its respective language. This includes
    /// many core types, like string types, basic container types, numbers, and other primitives.
    Special(SpecialType),
    /// A type with no generic parameters that is not considered a **special** type. This includes
    /// all user-generated types and some types from the standard library or third-party crates.
    /// However, these types can still be transformed as part of the type-map in `typeshare.toml`.
    Simple {
        #[allow(missing_docs)]
        id: String,
    },
}

impl Type {
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
        matches!(self, Self::Special(SpecialType::Option(_)))
    }

    /// Check if the type is `Option<Option<T>>`
    pub fn is_double_optional(&self) -> bool {
        match &self {
            Type::Special(SpecialType::Option(t)) => {
                matches!(t.as_ref(), Type::Special(SpecialType::Option(_)))
            }
            _ => false,
        }
    }
    /// Check if the type is `Vec<T>`
    pub fn is_vec(&self) -> bool {
        matches!(self, Self::Special(SpecialType::Vec(_)))
    }
    /// Check if the type is `HashMap<K, V>`
    pub fn is_hash_map(&self) -> bool {
        matches!(self, Self::Special(SpecialType::Map(_, _)))
    }
    /// Get the generic parameters for this type. Returns an empty iterator if there are none.
    /// For example, `Vec<String>`'s generic parameters would be `[String]`.
    /// Meanwhile, `HashMap<i64, u32>`'s generic parameters would be `[i64, u32]`.
    /// Finally, a type like `String` would have no generic parameters.
    ///
    // TODO remove dynamic dispatch
    pub fn parameters(&self) -> Box<dyn Iterator<Item = &Self> + '_> {
        match &self {
            Self::Simple { .. } => Box::new(std::iter::empty()),
            Self::Generic { parameters, .. } => Box::new(parameters.iter()),
            Self::Special(special) => special.parameters(),
        }
    }
}

/// A special rust type that needs a manual type conversion
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-everything",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde-everything", serde(tag = "type", content = "value"))]
pub enum SpecialType {
    /// Represents `Vec<T>` from the standard library
    Vec(Box<Type>),
    /// Represents `[T; N]` from the standard library
    Array(Box<Type>, usize),
    /// Represents `&[T]` from the standard library
    Slice(Box<Type>),
    /// Represents `HashMap<K, V>` from the standard library
    Map(Box<Type>, Box<Type>),
    /// Represents `Option<T>` from the standard library
    Option(Box<Type>),
    /// Represents `()`
    Unit,
    /// Represents `String` from the standard library
    String,
    /// Represents `char`
    Char,
    /// Represents a number type
    Number(Number),
    /// Represents `bool`
    Bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-everything",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum Number {
    /// Represents `i8`
    I8,
    /// Represents `i16`
    I16,
    /// Represents `i32`
    I32,
    /// Represents `i64`
    I64,
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
    /// Represents `f32`
    F32,
    /// Represents `f64`
    F64,
    /// Represents `I54` from `typeshare::I54`
    I54,
    /// Represents `U53` from `typeshare::U53`
    U53,
}
impl Into<SpecialType> for Number {
    fn into(self) -> SpecialType {
        SpecialType::Number(self)
    }
}
impl Number {
    pub fn id(&self) -> &'static str {
        match self {
            Self::F64 => "f64",
            Self::F32 => "f32",
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
        }
    }
}
impl SpecialType {
    /// Check if this type is equivalent to or contains `ty` in one of its generic parameters.
    pub fn contains_type(&self, ty: &str) -> bool {
        match &self {
            Self::Vec(rty) | Self::Array(rty, _) | Self::Slice(rty) | Self::Option(rty) => {
                rty.contains_type(ty)
            }
            Self::Map(rty1, rty2) => rty1.contains_type(ty) || rty2.contains_type(ty),
            Self::Unit | Self::String | Self::Char | Self::Bool => ty == self.id(),
            Self::Number(n) => ty == n.id(),
        }
    }

    /// Returns the Rust identifier for this special type.
    pub fn id(&self) -> &'static str {
        match &self {
            Self::Unit => "()",
            Self::Vec(_) => "Vec",
            Self::Array(_, _) => "[]",
            Self::Slice(_) => "&[]",
            Self::Option(_) => "Option",
            Self::Map(_, _) => "HashMap",
            Self::String => "String",
            Self::Char => "char",
            Self::Bool => "bool",
            Self::Number(n) => n.id(),
        }
    }
    /// Iterate over the generic parameters for this type. Returns an empty iterator
    /// if there are none.
    // TODO remove dynamic dispatch
    pub fn parameters(&self) -> Box<dyn Iterator<Item = &Type> + '_> {
        match &self {
            Self::Vec(rtype) | Self::Array(rtype, _) | Self::Slice(rtype) | Self::Option(rtype) => {
                Box::new(std::iter::once(rtype.as_ref()))
            }
            Self::Map(rtype1, rtype2) => Box::new([rtype1.as_ref(), rtype2.as_ref()].into_iter()),
            Self::Number(_) | Self::Unit | Self::String | Self::Char | Self::Bool => {
                Box::new(std::iter::empty())
            }
        }
    }
}
