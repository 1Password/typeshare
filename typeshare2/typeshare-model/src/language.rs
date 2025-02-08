use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    io::Write,
    path::Path,
};

use itertools::Itertools;

use crate::parsed_data::{
    CrateName, Id, ParsedData, RustEnum, RustEnumVariant, RustStruct, RustType, RustTypeAlias,
    SpecialRustType, TypeName,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FilesMode {
    Single,
    Multi,
}

/// Mapping of crate names to typeshare type names.
pub type CrateTypes = HashMap<CrateName, HashSet<TypeName>>;

/// Refence types by crate that are scoped for a given output module.
pub type ScopedCrateTypes<'a> = BTreeMap<&'a CrateName, BTreeSet<&'a TypeName>>;

/// Language-specific state and processing.
///
/// The `Language` implementation is allowed to maintain mutable state, and it
/// is allowed to assume that a unique `Language` instance will be constructed
/// for each `Generator` instance.
pub trait Language<'config>: Sized {
    /// Not all languages can format all types; the `format_` methods of this
    /// trait will return this error if an invalid type is encountered.
    // TODO: type formatting errors should include context during stack
    // unwinding
    type FormatTypeError;

    /// The configuration for this language. This configuration will be loaded
    /// from a config file and, where possible, from the command line. The way
    /// this works is sort of magical and will be extensively documented later;
    /// for now suffice it to say that we use serde shennanigans to make it work.
    ///
    /// The `Default` implementation ideally should have the "real" default
    /// values for this language, as it will be used to populate a config file,
    /// if the user so desires.
    ///
    /// The `serialize` implementation for this type should NOT skip keys, if
    /// possible.
    type Config: serde::Deserialize<'config> + serde::Serialize + crate::cli::ConfigCliArgs<'config>;

    /// The lowercase conventional name for this language. This should be a
    /// single identifier. It will be used as a prefix for various things;
    /// for instance, it will identify this language in the config file, and
    /// be used as a prefix when generating CLI parameters
    const NAME: &'static str;

    /// Create an instance of this language
    fn new_from_config(config: Self::Config) -> anyhow::Result<Self>;

    /// Most languages provide manual overrides for specific types.
    #[expect(unused_variables)]
    fn mapped_type(&self, type_name: &TypeName) -> Option<&TypeName> {
        None
    }

    /// In multi-file mode, typeshare will output one separate file with this
    /// name for each crate in the input set. These file names should have the
    /// appropriate naming convention and extension for this language.
    ///
    /// This method isn't used in single-file mode.
    fn output_file_for_crate(&self, crate_name: &CrateName) -> String;

    /// Convert a Rust type into a type from this language. By default this
    /// calls `format_simple_type`, `format_generic_type`
    fn format_type(
        &self,
        ty: &RustType,
        generic_context: &[TypeName],
    ) -> Result<String, Self::FormatTypeError> {
        match ty {
            RustType::Simple { id } => self.format_simple_type(id, generic_context),
            RustType::Generic { id, parameters } => {
                self.format_generic_type(id, parameters.as_slice(), generic_context)
            }
            RustType::Special(special) => self.format_special_type(special, generic_context),
        }
    }

    /// Format a simple type with no generic parameters.
    /// Note that we still need to take a list of generic types in case the implementors
    /// need to differentiate between a user-defined type and a generic type (for example: Swift)
    fn format_simple_type(
        &self,
        base: &TypeName,
        #[expect(unused_variables)] generic_context: &[TypeName],
    ) -> Result<String, Self::FormatTypeError> {
        Ok(match self.mapped_type(base) {
            Some(mapped) => mapped.to_string(),
            None => base.to_string(),
        })
    }

    /// Format a generic type that takes in generic arguments, which
    /// may be recursive.
    fn format_generic_type(
        &self,
        base: &TypeName,
        parameters: &[RustType],
        generic_context: &[TypeName],
    ) -> Result<String, Self::FormatTypeError> {
        match parameters.is_empty() {
            true => self.format_simple_type(base, generic_context),
            false => Ok(match self.mapped_type(base) {
                Some(mapped) => mapped.to_string(),
                None => format!(
                    "{}{}",
                    self.format_simple_type(base, generic_context)?,
                    self.format_generic_parameters(parameters, generic_context)?,
                ),
            }),
        }
    }

    /// Format generic parameters into a syntax used by this language. By
    /// default, this returns `<A, B, C, ...>`, since that's a common syntax
    /// used by most languages.
    fn format_generic_parameters(
        &self,
        parameters: &[RustType],
        generic_context: &[TypeName],
    ) -> Result<String, Self::FormatTypeError> {
        parameters
            .iter()
            .map(|ty| self.format_type(ty, generic_context))
            .process_results(|mut formatted| format!("<{}>", formatted.join(", ")))
    }

    /// Format a base type that is classified as a SpecialRustType.
    fn format_special_type(
        &self,
        special_ty: &SpecialRustType,
        generic_context: &[TypeName],
    ) -> Result<String, Self::FormatTypeError>;

    /// Implementors can use this function to write a header for typeshared code.
    /// By default this does nothing.
    #[expect(unused_variables)]
    fn begin_file(
        &self,
        w: &mut impl Write,
        parsed_data: &ParsedData,
        mode: FilesMode,
    ) -> std::io::Result<()> {
        Ok(())
    }

    /// For generating import statements. This is called only in multi-file
    /// mode, after `begin_file` and before any another methods.
    fn write_imports(
        &self,
        writer: &mut impl Write,
        imports: &ScopedCrateTypes<'_>,
    ) -> std::io::Result<()>;

    /// Implementors can use this function to write a footer for typeshared code.
    /// By default this does nothing.
    fn end_file(&self, _w: &mut impl Write) -> std::io::Result<()> {
        Ok(())
    }

    /// Write a type alias by converting it.
    /// Example of a type alias:
    /// ```
    /// type MyTypeAlias = String;
    /// ```
    fn write_type_alias(&self, w: &mut impl Write, t: &RustTypeAlias) -> std::io::Result<()>;

    /// Write a struct by converting it
    /// Example of a struct:
    /// ```ignore
    /// #[typeshare]
    /// #[derive(Serialize, Deserialize)]
    /// struct Foo {
    ///     bar: String
    /// }
    /// ```
    fn write_struct(&self, w: &mut impl Write, rs: &RustStruct) -> std::io::Result<()>;

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
    fn write_enum(&self, w: &mut impl Write, e: &RustEnum) -> std::io::Result<()>;

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
        &self,
        w: &mut impl Write,
        e: &RustEnum,
        make_struct_name: &impl Fn(&TypeName) -> String,
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
                        original: TypeName::new_string(struct_name.clone()),
                        renamed: TypeName::new_string(struct_name)
                    },
                    fields: fields.clone(),
                    generic_types,
                    comments: vec![format!(
                        "Generated type representing the anonymous struct variant `{}` of the `{}` Rust enum",
                        &shared.id.original,
                        &e.shared().id.original,
                    )],
                    decorators: e.shared().decorators.clone(),
                },
            )?;
        }

        Ok(())
    }

    /// Types that are remapped will be excluded from import references.
    fn ignored_reference_types(&self) -> Vec<&str> {
        Vec::new()
    }

    /// Any other final steps after modules have been generated. For example creating a new
    /// module with special types.
    // TODO: different error type
    fn post_generation(&self, _output_folder: &Path) -> std::io::Result<()> {
        Ok(())
    }
}
