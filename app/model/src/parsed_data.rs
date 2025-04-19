use std::{
    borrow::{Borrow, Cow},
    cmp::Ord,
    ffi::OsStr,
    fmt::{self, Display},
    path::{Component, Path},
};

use crate::decorator::DecoratorSet;

/// A crate name.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct CrateName(String);

impl Display for CrateName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl CrateName {
    pub const fn new(name: String) -> Self {
        Self(name)
    }

    /// View this crate name as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Extract the crate name from a give path to a rust source file. This is
    /// defined as the name of the directory one level above the `src` directory
    /// that cotains this source file, with any `-` replaced with `_`.
    pub fn find_crate_name(path: &Path) -> Option<Self> {
        path.ancestors()
            // Only consider paths that contain normal stuff. If there's a
            // .. or anything like that, skip it.
            .take_while(|path| {
                matches!(
                    path.components().next_back(),
                    Some(Component::Normal(_) | Component::CurDir)
                )
            })
            // Find the `src directory`
            .find(|path| path.file_name() == Some(OsStr::new("src")))?
            // The directory that contains it is the crate name candidate
            .parent()?
            // Get the crate name and convert it to a string, with - replaced
            .file_name()?
            .to_str()
            .map(|name| name.replace("-", "_"))
            .map(CrateName)
    }
}

impl PartialEq<str> for CrateName {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for CrateName {
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

/// Identifier used in Rust structs, enums, and fields. It includes the
/// `original` name and the `renamed` value after the transformation based on `serde` attributes.
#[derive(Debug, Clone)]
pub struct Id {
    /// The original identifier name
    pub original: TypeName,
    /// The renamed identifier, based on serde attributes.
    /// If there is no re-naming going on, this will be identical to
    /// `original`.
    pub renamed: TypeName,
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
    pub generic_types: Vec<TypeName>,
    /// The fields of the struct.
    pub fields: Vec<RustField>,
    /// Comments that were in the struct source.
    /// We copy comments over to the typeshared files,
    /// so we need to collect them here.
    pub comments: Vec<String>,
    /// Attributes that exist for this struct.
    pub decorators: DecoratorSet,
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
    pub generic_types: Vec<TypeName>,
    /// The type identifier that this type alias is aliasing
    pub ty: RustType,
    /// Comments that were in the type alias source.
    pub comments: Vec<String>,
    /// Attributes that exist for this struct.
    pub decorators: DecoratorSet,
}

/// Rust field definition.
#[derive(Debug, Clone)]
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
    pub decorators: DecoratorSet,
}

/// A named Rust type.
#[derive(Debug, Clone)]
pub enum RustType {
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
        id: TypeName,
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
        id: TypeName,
    },
}

/// A special rust type that needs a manual type conversion
#[derive(Debug, Clone)]
#[non_exhaustive]
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

impl RustType {
    /// Check if a type contains a type with an ID that matches `ty`.
    /// For example, `Box<String>` contains the types `Box` and `String`. Similarly,
    /// `Vec<Option<HashMap<String, Url>>>` contains the types `Vec`, `Option`, `HashMap`,
    /// `String`, and `Url`.
    pub fn contains_type(&self, ty: &TypeName) -> bool {
        match &self {
            Self::Simple { id } => id == ty,
            Self::Generic { id, parameters } => {
                id == ty || parameters.iter().any(|p| p.contains_type(ty))
            }
            Self::Special(special) => special.contains_type(ty),
        }
    }

    /// Get the ID (AKA name) of the type. Special types don't have an ID.
    pub fn id(&self) -> Option<&TypeName> {
        match &self {
            Self::Simple { id } | Self::Generic { id, .. } => Some(id),
            Self::Special(_) => None,
        }
    }
    /// Check if the type is `Option<T>`
    pub fn is_optional(&self) -> bool {
        matches!(self, Self::Special(SpecialRustType::Option(_)))
    }

    /// Check if the type is `Option<Option<T>>`
    pub fn is_double_optional(&self) -> bool {
        matches!(self, Self::Special(SpecialRustType::Option(inner)) if inner.is_optional())
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
}

impl SpecialRustType {
    /// Check if this type is equivalent to or contains `ty` in one of its generic parameters.
    /// This only operates on "externally named" types; (that is, types that aren't primitives)
    /// because it's used only to detect the presence of generics, or that a type is recursive,
    /// or anything like that. It always returns false for things like ints and strings.
    pub fn contains_type(&self, ty: &TypeName) -> bool {
        match self {
            Self::Vec(rty) | Self::Array(rty, _) | Self::Slice(rty) | Self::Option(rty) => {
                rty.contains_type(ty)
            }
            Self::HashMap(rty1, rty2) => rty1.contains_type(ty) || rty2.contains_type(ty),

            // Comprehensive list to ensure this is updated if new types are
            // added
            Self::Unit
            | Self::String
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
            | Self::U53 => false,
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
    Unit {
        /// Shared context for this enum
        shared: RustEnumShared,

        /// All of the variants for this enum. This is a Unit enum, so all
        /// of these variants have only unit data available.
        unit_variants: Vec<RustEnumVariantShared>,
    },

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
        /// The variants on this enum
        variants: Vec<RustEnumVariant>,
    },
}

impl RustEnum {
    /// Get a reference to the inner shared content
    pub fn shared(&self) -> &RustEnumShared {
        match self {
            Self::Unit { shared, .. } | Self::Algebraic { shared, .. } => shared,
        }
    }
}

/// Enum information shared among different enum types
#[derive(Debug, Clone)]
pub struct RustEnumShared {
    /// The enum's ident
    pub id: Id,
    /// Generic parameters for the enum, e.g. `SomeEnum<T>` would produce `vec!["T"]`
    pub generic_types: Vec<TypeName>,
    /// Comments on the enum definition itself
    pub comments: Vec<String>,

    /// Decorators applied to the enum for generation in other languages
    ///
    /// Example: `#[typeshare(swift = "Equatable, Comparable, Hashable")]`.
    pub decorators: DecoratorSet,
    /// True if this enum references itself in any field of any variant
    /// Swift needs the special keyword `indirect` for this case
    pub is_recursive: bool,
}

/// Parsed information about a Rust enum variant
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum RustEnumVariant {
    /// A unit variant
    Unit(RustEnumVariantShared),
    /// A newtype tuple variant
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
#[derive(Debug, Clone)]
pub struct RustEnumVariantShared {
    /// The variant's ident
    pub id: Id,
    /// Comments applied to the variant
    pub comments: Vec<String>,
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
    pub ty: RustType,
    /// The expression that the constant contains.
    pub expr: RustConstExpr,
}

/// A constant expression that can be shared via a constant variable across the typeshare
/// boundary.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum RustConstExpr {
    /// Expression represents an integer.
    Int(i128),
}

/// An imported type reference.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImportedType {
    /// Crate this type belongs to.
    pub base_crate: CrateName,
    /// Type name.
    pub type_name: TypeName,
}

// TODO: replace this `Cow` with a pair of owned/borrowed types
/// A type name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TypeName(Cow<'static, str>);

impl TypeName {
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    #[inline]
    #[must_use]
    pub fn new_string(ident: String) -> Self {
        Self(Cow::Owned(ident))
    }

    #[inline]
    #[must_use]
    pub const fn new_static(ident: &'static str) -> Self {
        Self(Cow::Borrowed(ident))
    }
}

impl AsRef<str> for TypeName {
    #[inline]
    #[must_use]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for TypeName {
    #[inline]
    #[must_use]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for TypeName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl PartialEq<str> for TypeName {
    #[inline]
    #[must_use]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for TypeName {
    #[inline]
    #[must_use]
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

#[cfg(test)]
mod test {
    use super::CrateName;
    use std::path::Path;

    #[test]
    fn test_crate_name() {
        let path = Path::new("/some/path/to/projects/core/foundation/op-proxy/src/android.rs");
        assert_eq!(CrateName::find_crate_name(path).unwrap(), "op_proxy",);
    }

    #[test]
    fn skip_curdir() {
        let path = Path::new("/path/to/crate-name/./src/main.rs");
        assert_eq!(CrateName::find_crate_name(path).unwrap(), "crate_name")
    }

    #[test]
    fn bail_on_parent_dir() {
        let path = Path::new("/path/to/crate-name/src/foo/../stuff.rs");
        assert!(CrateName::find_crate_name(path).is_none());
    }

    #[test]
    fn accept_parent_dir_before_crate() {
        let path = Path::new("/path/to/../crate/src/foo/bar/stuff.rs");
        assert_eq!(CrateName::find_crate_name(path).unwrap(), "crate");
    }

    #[test]
    fn reject_rooted_src() {
        let path = Path::new("/src/foo.rs");
        assert!(CrateName::find_crate_name(path).is_none());
    }
}

#[cfg(test)]
mod rust_type_api {
    use super::*;

    const INT: RustType = RustType::Special(SpecialRustType::I32);

    fn make_option(inner: RustType) -> RustType {
        RustType::Special(SpecialRustType::Option(Box::new(inner)))
    }

    #[test]
    fn test_optional() {
        let ty = make_option(INT);

        assert!(ty.is_optional());
        assert!(!ty.is_double_optional());
        assert!(!ty.is_hash_map());
    }

    #[test]
    fn test_double_optional() {
        let ty = make_option(make_option(INT));

        assert!(ty.is_optional());
        assert!(ty.is_double_optional());
        assert!(!ty.is_vec());
    }
}
