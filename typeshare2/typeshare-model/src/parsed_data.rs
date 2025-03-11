use std::{
    borrow::{Borrow, Cow},
    cmp::Ord,
    collections::HashSet,
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
        path.components()
            .rev()
            // Only find paths that use normal components in the suffix. If we
            // hit something like `..` or `C:\`, end the search immediately.
            .take_while(|c| matches!(c, Component::Normal(_) | Component::CurDir))
            // Skip `.` paths entirely
            .filter_map(|c| match c {
                Component::Normal(name) => Some(name),
                _ => None,
            })
            // Find the `src` directory in our ancestors
            .skip_while(|&name| name != "src")
            // Find the first directory preceeding the `src` directory
            .find(|&name| name != "src")?
            // Convert this directory name to a string; fail if it isn't
            // stringable
            .to_str()
            // Fix dashes
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
    /// TODO: This is currently almost empty. Import computation was found to
    /// be pretty broken during the migration to Typeshare 2, so that part
    /// of multi-file output was stripped out to be restored later.
    pub import_types: HashSet<ImportedType>,
}

impl ParsedData {
    pub fn merge(&mut self, other: Self) {
        self.structs.extend(other.structs);
        self.enums.extend(other.enums);
        self.aliases.extend(other.aliases);
        self.import_types.extend(other.import_types);
    }

    pub fn add(&mut self, item: RustItem) {
        match item {
            RustItem::Struct(rust_struct) => self.structs.push(rust_struct),
            RustItem::Enum(rust_enum) => self.enums.push(rust_enum),
            RustItem::Alias(rust_type_alias) => self.aliases.push(rust_type_alias),
        }
    }

    pub fn all_type_names(&self) -> impl Iterator<Item = &'_ TypeName> + use<'_> {
        let s = self.structs.iter().map(|s| &s.id.renamed);
        let e = self.enums.iter().map(|e| &e.shared().id.renamed);
        let a = self.aliases.iter().map(|a| &a.id.renamed);

        s.chain(e).chain(a)
    }

    pub fn sort_contents(&mut self) {
        self.structs
            .sort_unstable_by(|lhs, rhs| Ord::cmp(&lhs.id.original, &rhs.id.original));

        self.enums.sort_unstable_by(|lhs, rhs| {
            Ord::cmp(&lhs.shared().id.original, &rhs.shared().id.original)
        });

        self.aliases
            .sort_unstable_by(|lhs, rhs| Ord::cmp(&lhs.id.original, &rhs.id.original));
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LangIdent(String);

impl Borrow<str> for LangIdent {
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}

/// Identifier used in Rust structs, enums, and fields. It includes the `original` name and the `renamed` value after the transformation based on `serde` attributes.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
pub struct RustTypeAlias {
    /// The identifier for the alias.
    pub id: Id,
    /// The generic parameters that come after the type alias name.
    pub generic_types: Vec<TypeName>,
    /// The type identifier that this type alias is aliasing
    pub r#type: RustType,
    /// Comments that were in the type alias source.
    pub comments: Vec<String>,
}

/// Rust field definition.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// Get the ID (AKA name) of the type.
    pub fn id(&self) -> &TypeName {
        match &self {
            Self::Simple { id } | Self::Generic { id, .. } => id,
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

    // /// Yield all the type names including nested generic types.
    // pub fn all_reference_type_names(&self) -> impl Iterator<Item = &'_ str> + '_ {
    //     RustRefTypeIter {
    //         ty: Some(self),
    //         parameters: Vec::new(),
    //     }
    //     .filter(|s| accept_type(s))
    // }
}

impl SpecialRustType {
    /// Check if this type is equivalent to or contains `ty` in one of its generic parameters.
    pub fn contains_type(&self, ty: &TypeName) -> bool {
        match self {
            Self::Vec(rty) | Self::Array(rty, _) | Self::Slice(rty) | Self::Option(rty) => {
                rty.contains_type(ty)
            }
            Self::HashMap(rty1, rty2) => rty1.contains_type(ty) || rty2.contains_type(ty),
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
            | Self::U53 => ty == self.id(),
        }
    }

    /// Returns the Rust identifier for this special type.
    pub const fn id(&self) -> &'static TypeName {
        // Helper macro to handle the tedium of repeating the `const` block
        // in each match arm
        macro_rules! match_block {
            {
                match $this:ident {
                    $($pattern:pat => $out:literal,)*
                }
            } => {
                match $this {
                    $($pattern => const {&TypeName(Cow::Borrowed($out))},)*
                }
            }
        }

        // TODO: I suspect there are bugs related to strings like `[]` being
        // returned from this function (non-identifier strings) but it seems
        // to work fine so I'll leave it for now.
        match_block! {
            match self {
                Self::Unit => "()",
                Self::F64 => "f64",
                Self::F32 => "f32",
                Self::Vec(_) => "Vec",
                Self::Array(_, _) => "[]",
                Self::Slice(_) => "&[]",
                Self::Option(_) => "Option",
                Self::HashMap(_, _) => "HashMap",
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
            }
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
#[derive(Debug, Clone, PartialEq)]
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
    pub generic_types: Vec<TypeName>,
    /// Comments on the enum definition itself
    pub comments: Vec<String>,
    /// The enum's variants
    pub variants: Vec<RustEnumVariant>,
    /// Decorators applied to the enum for generation in other languages
    ///
    /// Example: `#[typeshare(swift = "Equatable, Comparable, Hashable")]`.
    pub decorators: DecoratorSet,
    /// True if this enum references itself in any field of any variant
    /// Swift needs the special keyword `indirect` for this case
    pub is_recursive: bool,
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
    pub fn new(ident: &proc_macro2::Ident) -> Self {
        Self::new_string(ident.to_string())
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
