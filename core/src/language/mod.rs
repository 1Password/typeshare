use crate::{
    parser::{ParseError, ParsedData},
    rust_types::{Id, RustEnum, RustEnumVariant, RustItem, RustStruct, RustTypeAlias},
    topsort::topsort,
};
use itertools::Itertools;
use proc_macro2::Ident;
use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet, HashMap},
    io::Write,
    str::FromStr,
};

mod go;
mod kotlin;
mod scala;
mod swift;
mod typescript;

use crate::rust_types::{RustType, RustTypeFormatError, SpecialRustType};
pub use go::Go;
pub use kotlin::Kotlin;
pub use scala::Scala;
pub use swift::GenericConstraints;
pub use swift::Swift;
pub use typescript::TypeScript;

/// All supported programming languages.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SupportedLanguage {
    Go,
    Kotlin,
    Scala,
    Swift,
    TypeScript,
}

impl SupportedLanguage {
    /// Returns an iterator over all supported language variants.
    pub fn all_languages() -> impl Iterator<Item = Self> {
        use SupportedLanguage::*;
        [Go, Kotlin, Scala, Swift, TypeScript].into_iter()
    }
}

impl FromStr for SupportedLanguage {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "go" => Ok(Self::Go),
            "kotlin" => Ok(Self::Kotlin),
            "scala" => Ok(Self::Scala),
            "swift" => Ok(Self::Swift),
            "typescript" => Ok(Self::TypeScript),
            _ => Err(ParseError::UnsupportedLanguage(s.into())),
        }
    }
}

impl TryFrom<&Ident> for SupportedLanguage {
    type Error = ParseError;

    fn try_from(ident: &Ident) -> Result<Self, Self::Error> {
        Self::from_str(ident.to_string().as_str())
    }
}

/// Language-specific state and processing.
///
/// The `Language` implementation is allowed to maintain mutable state, and it
/// is allowed to assume that a unique `Language` instance will be constructed
/// for each `Generator` instance.
pub trait Language {
    /// Given `data`, generate type-code for this language and write it out to `writable`.
    /// Returns whether or not writing was successful.
    fn generate_types(
        &mut self,
        writable: &mut dyn Write,
        all_types: &HashMap<String, Vec<String>>,
        data: &ParsedData,
    ) -> std::io::Result<()> {
        self.begin_file(writable, data)?;

        self.write_imports(writable, used_imports(data, all_types))?;

        let mut items: Vec<RustItem> = vec![];

        for a in &data.aliases {
            items.push(RustItem::Alias(a.clone()))
        }

        for s in &data.structs {
            items.push(RustItem::Struct(s.clone()))
        }

        for e in &data.enums {
            items.push(RustItem::Enum(e.clone()))
        }

        let sorted = topsort(items.iter().collect());

        for &thing in &sorted {
            match thing {
                RustItem::Enum(e) => self.write_enum(writable, e)?,
                RustItem::Struct(s) => self.write_struct(writable, s)?,
                RustItem::Alias(a) => self.write_type_alias(writable, a)?,
            }
        }

        self.end_file(writable)?;

        Ok(())
    }

    /// Get the type mapping for this language `(Rust type name -> lang type name)`
    fn type_map(&mut self) -> &HashMap<String, String>;

    /// Convert a Rust type into a type from this language.
    fn format_type(
        &mut self,
        ty: &RustType,
        generic_types: &[String],
    ) -> Result<String, RustTypeFormatError> {
        match ty {
            RustType::Simple { id } => self.format_simple_type(id, generic_types),
            RustType::Generic { id, parameters } => {
                self.format_generic_type(id, parameters.as_slice(), generic_types)
            }
            RustType::Special(special) => self.format_special_type(special, generic_types),
        }
    }

    // We need to pass in an &String for type mapping
    /// Format a simple type with no generic parameters.
    /// Note that we still need to take a list of generic types in case the implementors
    /// need to differentiate between a user-defined type and a generic type (for example: Swift)
    #[allow(clippy::ptr_arg)]
    fn format_simple_type(
        &mut self,
        base: &String,
        _generic_types: &[String],
    ) -> Result<String, RustTypeFormatError> {
        Ok(if let Some(mapped) = self.type_map().get(base) {
            mapped.into()
        } else {
            base.into()
        })
    }

    // We need to pass in an &String for type mapping
    /// Format a generic type that takes in generic arguments, which
    /// may be recursive.
    #[allow(clippy::ptr_arg)]
    fn format_generic_type(
        &mut self,
        base: &String,
        parameters: &[RustType],
        generic_types: &[String],
    ) -> Result<String, RustTypeFormatError> {
        if let Some(mapped) = self.type_map().get(base) {
            Ok(mapped.into())
        } else {
            let parameters: Result<Vec<String>, RustTypeFormatError> = parameters
                .iter()
                .map(|p| self.format_type(p, generic_types))
                .collect();
            let parameters = parameters?;
            Ok(format!(
                "{}{}",
                self.format_simple_type(base, generic_types)?,
                (!parameters.is_empty())
                    .then(|| self.format_generic_parameters(parameters))
                    .unwrap_or_default()
            ))
        }
    }

    /// Format generic parameters into A<T,R> which is common for many supported languages.
    /// Reimplement if other notations is used.
    fn format_generic_parameters(&mut self, parameters: Vec<String>) -> String {
        format!("<{}>", parameters.into_iter().join(", "))
    }

    /// Format a base type that is classified as a SpecialRustType.
    fn format_special_type(
        &mut self,
        special_ty: &SpecialRustType,
        generic_types: &[String],
    ) -> Result<String, RustTypeFormatError>;

    /// Implementors can use this function to write a header for typeshared code
    fn begin_file(&mut self, _w: &mut dyn Write, _parsed_data: &ParsedData) -> std::io::Result<()> {
        Ok(())
    }

    /// For generating import statements.
    fn write_imports(
        &mut self,
        _writer: &mut dyn Write,
        _imports: BTreeMap<&str, BTreeSet<&str>>,
    ) -> std::io::Result<()> {
        Ok(())
    }

    /// Implementors can use this function to write a footer for typeshared code
    fn end_file(&mut self, _w: &mut dyn Write) -> std::io::Result<()> {
        Ok(())
    }

    /// Write a type alias by converting it.
    /// Example of a type alias:
    /// ```
    /// type MyTypeAlias = String;
    /// ```
    fn write_type_alias(&mut self, _w: &mut dyn Write, _t: &RustTypeAlias) -> std::io::Result<()> {
        Ok(())
    }

    /// Write a struct by converting it
    /// Example of a struct:
    /// ```ignore
    /// #[typeshare]
    /// #[derive(Serialize, Deserialize)]
    /// struct Foo {
    ///     bar: String
    /// }
    /// ```
    fn write_struct(&mut self, _w: &mut dyn Write, _rs: &RustStruct) -> std::io::Result<()> {
        Ok(())
    }

    /// Write an enum by converting it.
    /// Example of an enum:
    /// ```ignore
    /// #[typeshare]
    /// #[derive(Serialize, Deserialize)]
    /// #[serde(tag = "type", content = "content")]
    /// enum Foo {
    ///     Fizz,
    ///     Buzz { yep_this_works: bool }
    /// }
    /// ```
    fn write_enum(&mut self, _w: &mut dyn Write, _e: &RustEnum) -> std::io::Result<()> {
        Ok(())
    }

    /// Write out named types to represent anonymous struct enum variants.
    ///
    /// Take the following enum as an example:
    ///
    /// ```
    /// enum AlgebraicEnum {
    ///     AnonymousStruct {
    ///         field: String,
    ///         another_field: bool,
    ///     },
    /// }
    /// ```
    ///
    /// This function will write out:
    ///
    /// ```compile_fail
    /// /// Generated type representing the anonymous struct variant `<make_struct_name>` of the `AlgebraicEnum` rust enum
    /// /* the struct definition for whatever language */
    /// ```
    ///
    /// It does this by calling `write_struct` on the given `language_impl`. The
    /// name of the struct is controlled by the `make_struct_name` closure; you're
    /// given the variant name and asked to return whatever struct name works best
    /// for your language.
    fn write_types_for_anonymous_structs(
        &mut self,
        w: &mut dyn Write,
        e: &RustEnum,
        make_struct_name: &dyn Fn(&str) -> String,
    ) -> std::io::Result<()> {
        for (fields, shared) in e.shared().variants.iter().filter_map(|v| match v {
            RustEnumVariant::AnonymousStruct { fields, shared } => Some((fields, shared)),
            _ => None,
        }) {
            let struct_name = make_struct_name(&shared.id.original);

            // Builds the list of generic types (e.g [T, U, V]), by digging
            // through the fields recursively and comparing against the
            // enclosing enum's list of generic parameters.
            let generic_types = fields
                .iter()
                .flat_map(|field| {
                    e.shared()
                        .generic_types
                        .iter()
                        .filter(|g| field.ty.contains_type(g))
                })
                .unique()
                .cloned()
                .collect();

            self.write_struct(
                w,
                &RustStruct {
                    id: Id {
                        original: struct_name.clone(),
                        renamed: struct_name.clone(),
                    },
                    fields: fields.clone(),
                    generic_types,
                    comments: vec![format!(
                        "Generated type representing the anonymous struct variant `{}` of the `{}` Rust enum",
                        &shared.id.original,
                        &e.shared().id.original,
                    )],
                    decorators: HashMap::new(),
                },
            )?;
        }

        Ok(())
    }
}

/// Lookup any refeferences to other typeshared types in order to build
/// a list of imports for the generated module.
fn used_imports<'a, 'b: 'a>(
    data: &'b ParsedData,
    all_types: &'a HashMap<String, Vec<String>>,
) -> BTreeMap<&'a str, BTreeSet<&'a str>> {
    let mut used_imports: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();

    for referenced_import in data.import_types.iter() {
        // Skip over imports that reference the current crate. They
        // are all collapsed into one module per crate.
        if data.crate_name == referenced_import.base_crate {
            continue;
        }

        // Look up the types for the referenced imported crate.
        if let Some(type_names) = all_types.get(&referenced_import.base_crate) {
            // We can have "*" wildcard here. We need to add all.
            if referenced_import.type_name == "*" {
                used_imports
                    .entry(referenced_import.base_crate.as_str())
                    .and_modify(|names| names.extend(type_names.iter().map(|s| s.as_str())));
                continue;
            }

            // Add referenced import for each matching type.
            if let Some(ty_name) = type_names
                .iter()
                .find(|&t| t == &referenced_import.type_name)
            {
                match used_imports.entry(referenced_import.base_crate.as_str()) {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().insert(ty_name.as_str());
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(BTreeSet::from([ty_name.as_str()]));
                    }
                }
            }
        }
    }
    used_imports
}
