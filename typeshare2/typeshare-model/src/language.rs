use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    io::Write,
    path::Path,
};

use itertools::Itertools;

use crate::parsed_data::{
    CrateName, Id, ImportedType, ParsedData, RustEnum, RustEnumVariant, RustItem, RustStruct,
    RustType, RustTypeAlias, SpecialRustType, TypeName,
};
use crate::topsort::topsort;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FilesMode {
    Single,
    Multi,
}

/// Mapping of crate names to typeshare type names.
pub type CrateTypes = HashMap<CrateName, HashSet<TypeName>>;

/// A sorted crate name ref.
pub type SortedCrateNames<'a> = &'a CrateName;
/// A sorted type name ref.
pub type SortedTypeNames<'a> = BTreeSet<&'a str>;

/// Refence types by crate that are scoped for a given output module.
pub type ScopedCrateTypes<'a> = BTreeMap<SortedCrateNames<'a>, SortedTypeNames<'a>>;

/// Language-specific state and processing.
///
/// The `Language` implementation is allowed to maintain mutable state, and it
/// is allowed to assume that a unique `Language` instance will be constructed
/// for each `Generator` instance.
pub trait Language {
    /// Not all languages can format all types; the `format_` methods of this
    /// trait will return this error if an invalid type is encountered.
    // TODO: type formatting errors should include context during stack
    // unwinding
    type FormatTypeError;

    /// Given `data`, generate type-code for this language and write it out to `writable`.
    /// Returns whether or not writing was successful.
    fn generate_types(
        &mut self,
        writable: &mut impl Write,
        all_types: &CrateTypes,
        data: ParsedData,
        mode: FilesMode,
    ) -> std::io::Result<()> {
        self.begin_file(writable, &data)?;

        if matches!(mode, FilesMode::Multi) {
            self.write_imports(writable, &used_imports(&data, all_types))?;
        }

        let ParsedData {
            structs,
            enums,
            aliases,
            ..
        } = data;

        let mut items = Vec::from_iter(
            aliases
                .into_iter()
                .map(RustItem::Alias)
                .chain(structs.into_iter().map(RustItem::Struct))
                .chain(enums.into_iter().map(RustItem::Enum)),
        );

        topsort(&mut items);

        for thing in &items {
            match thing {
                RustItem::Enum(e) => self.write_enum(writable, e)?,
                RustItem::Struct(s) => self.write_struct(writable, s)?,
                RustItem::Alias(a) => self.write_type_alias(writable, a)?,
            }
        }

        self.end_file(writable)
    }

    fn mapped_type(&self, #[expect(unused_variables)] type_name: &TypeName) -> Option<&TypeName> {
        None
    }

    /// Convert a Rust type into a type from this language.
    fn format_type(
        &mut self,
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
        &mut self,
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
        &mut self,
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
        &mut self,
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
        &mut self,
        special_ty: &SpecialRustType,
        generic_context: &[TypeName],
    ) -> Result<String, Self::FormatTypeError>;

    /// Implementors can use this function to write a header for typeshared code.
    /// By default this does nothing.
    fn begin_file(
        &mut self,
        _w: &mut impl Write,
        _parsed_data: &ParsedData,
    ) -> std::io::Result<()> {
        Ok(())
    }

    /// For generating import statements. By default, this is only called by
    /// `generate_types` in multi-file mode.
    fn write_imports(
        &mut self,
        writer: &mut impl Write,
        imports: &ScopedCrateTypes<'_>,
    ) -> std::io::Result<()>;

    /// Implementors can use this function to write a footer for typeshared code.
    /// By default this does nothing.
    fn end_file(&mut self, _w: &mut impl Write) -> std::io::Result<()> {
        Ok(())
    }

    /// Write a type alias by converting it.
    /// Example of a type alias:
    /// ```
    /// type MyTypeAlias = String;
    /// ```
    fn write_type_alias(&mut self, w: &mut impl Write, t: &RustTypeAlias) -> std::io::Result<()>;

    /// Write a struct by converting it
    /// Example of a struct:
    /// ```ignore
    /// #[typeshare]
    /// #[derive(Serialize, Deserialize)]
    /// struct Foo {
    ///     bar: String
    /// }
    /// ```
    fn write_struct(&mut self, w: &mut impl Write, rs: &RustStruct) -> std::io::Result<()>;

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
    fn write_enum(&mut self, w: &mut impl Write, e: &RustEnum) -> std::io::Result<()>;

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

/// Lookup any refeferences to other typeshared types in order to build
/// a list of imports for the generated module.
fn used_imports<'a, 'b: 'a>(
    data: &'b ParsedData,
    all_types: &'a CrateTypes,
) -> ScopedCrateTypes<'a> {
    let mut used_imports = BTreeMap::new();

    // If we have reference that is a re-export we can attempt to find it with the
    // following heuristic.
    let fallback = |referenced_import: &'a ImportedType, used: &mut ScopedCrateTypes<'a>| {
        // Find the first type that does not belong to the current crate.
        if let Some((crate_name, ty)) = all_types
            .iter()
            .flat_map(|(k, v)| {
                v.iter()
                    .find(|&t| *t == referenced_import.type_name && k != &data.crate_name)
                    .map(|t| (k, t))
            })
            .next()
        {
            println!("Warning: Using {crate_name} as module for {ty} which is not in referenced crate {}", referenced_import.base_crate);
            used.entry(crate_name)
                .and_modify(|v| {
                    v.insert(ty.as_str());
                })
                .or_insert(BTreeSet::from([ty.as_str()]));
        } else {
            // println!("Could not lookup reference {referenced_import:?}");
        }
    };

    for referenced_import in data
        .import_types
        .iter()
        // Skip over imports that reference the current crate. They
        // are all collapsed into one module per crate.
        .filter(|imp| imp.base_crate != data.crate_name)
    {
        // Look up the types for the referenced imported crate.
        if let Some(type_names) = all_types.get(&referenced_import.base_crate) {
            if referenced_import.type_name == "*" {
                // We can have "*" wildcard here. We need to add all.
                used_imports
                    .entry(&referenced_import.base_crate)
                    .and_modify(|names: &mut BTreeSet<&str>| {
                        names.extend(type_names.iter().map(|s| s.as_str()))
                    });
            } else if let Some(ty_name) = type_names.get(&referenced_import.type_name) {
                // Add referenced import for each matching type.
                used_imports
                    .entry(&referenced_import.base_crate)
                    .and_modify(|v| {
                        v.insert(ty_name.as_str());
                    })
                    .or_insert(BTreeSet::from([ty_name.as_str()]));
            } else {
                fallback(referenced_import, &mut used_imports);
            }
        } else {
            // We might have a re-export from another crate.
            fallback(referenced_import, &mut used_imports);
        }
    }
    used_imports
}
