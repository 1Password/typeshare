use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    io::Write,
    path::Path,
};

use itertools::Itertools;

use crate::parsed_data::{
    CrateName, Id, RustConst, RustEnum, RustEnumVariant, RustStruct, RustType, RustTypeAlias,
    SpecialRustType, TypeName,
};

/// If we're in multifile mode, this enum contains the crate name for the
/// specific file
#[derive(Debug, Clone, Copy)]
pub enum FilesMode<T> {
    Single,
    Multi(T),
}

impl<T> FilesMode<T> {
    pub fn map<U>(self, op: impl FnOnce(T) -> U) -> FilesMode<U> {
        match self {
            FilesMode::Single => FilesMode::Single,
            FilesMode::Multi(value) => FilesMode::Multi(op(value)),
        }
    }

    pub fn is_multi(&self) -> bool {
        matches!(*self, Self::Multi(_))
    }
}

/// Mapping of crate names to typeshare type names.
pub type CrateTypes = HashMap<CrateName, HashSet<TypeName>>;

/// Refence types by crate that are scoped for a given output module.
pub type ScopedCrateTypes<'a> = BTreeMap<&'a CrateName, BTreeSet<&'a TypeName>>;

/**
Language-specific state and processing.

The `Language` implementation is allowed to maintain mutable state, and it
is allowed to assume that a unique `Language` instance will be constructed
for each `Generator` instance.

The basic flow of `typeshare` looks like this. In this example we'll use
`Kotlin` as the example language (so we assume that somewhere there exists
`impl Language<'_> for Kotlin`).

1. The language's config is loaded from the config file and command line
arguments:

```ignore
let config = Kotlin::Config::deserialize(config_file);
```

2. The language is loaded from the config file via `new_from_config`. This is
where the implementation has the opportunity to report any configuration errors
that weren't detected during deserialization.

```ignore
let language = Kotlin::new_from_config(config)?;
```

3. If we're in multi-file mode, we call `output_file_for_crate` for each rust
crate being typeshared to determine the _filename_ for the output file that
will contain that crate's types.

```ignore
let files = crate_names
    .iter()
    .map(|crate_name| {
        let filename = language.output_file_for_type(crate_name);
        File::create(output_directory.join(filename))
    });
}
```

4. We call `begin_file` on the output type to print any headers or preamble
appropriate for this language. In multi-file mode, `begin_file` is called once
for each output file; in this case, the `mode` argument will include the crate
name.

```ignore
language.begin_file(&mut file, mode)
```

5. In mutli-file mode only, we call `write_imports` with a list of all the
types that are being imported from other typeshare'd crates. This allows the
language to emit appropriate import statements for its own language.

```ignore
language.write_imports(&mut file, crate_name, computed_imports)
```

6. For EACE typeshared item in being typeshared, we call `write_enum`,
`write_struct`, or `write_type_alias`, as appropriate.

```ignore
language.write_struct(&mut file, parsed_struct);
language.write_enum(&mut file, parsed_enum);
```

5a. In your implementations of these methods, we recommend that you call
`format_type` for the fields of these types. `format_type` will in turn call
`format_simple_type`, `format_generic_type`, or `format_special_type`, as
appropriate; usually it is only necessary for you to implmenent
`format_special_type` yourself, and use the default implementations for the
others. The `format_*` methods will otherwise never be called by typeshare.

5b. If your language doesn't natively support data-containing enums, we
recommand that you call `write_types_for_anonymous_structs` in your
implementation of `write_enum`; this will call `write_struct` for each variant
of the enum.

7. After all the types are written, we call `end_file`.

```ignore
language.end_file(&mut file)
```

8. In multi-file mode only, after ALL files are written, we call
`post_generation` with the output directory. This gives the language an
opportunity to create any files resembling `mod.rs` or `index.js` as it might
require.

```ignore
language.post_generation(&output_directory)
```

NOTE THAT at this stage, multi-file output is still work-in-progress, as the
algorithms that compute import sets are being rewritten.
*/
pub trait Language<'config>: Sized {
    /// Not all languages can format all types; the `format_` methods of this
    /// trait will return this error if an invalid type is encountered.
    // TODO: type formatting errors should include context during stack
    // unwinding
    type FormatTypeError;

    /// The configuration for this language. This configuration will be loaded
    /// from a config file and, where possible, from the command line, via
    /// `serde`.
    ///
    /// It is important that this type include `#[serde(default)]` or something
    /// equivelent, so that a config can be loaded with default setting even
    /// if this language isn't present in the config file.
    ///
    /// The `serialize` implementation for this type should NOT skip keys, if
    /// possible.
    type Config: serde::Deserialize<'config> + serde::Serialize;

    /// The lowercase conventional name for this language. This should be a
    /// single identifier. It will be used as a prefix for various things;
    /// for instance, it will identify this language in the config file, and
    /// be used as a prefix when generating CLI parameters
    const NAME: &'static str;

    /// Create an instance of this language
    fn new_from_config(config: Self::Config) -> anyhow::Result<Self>;

    /// Most languages provide manual overrides for specific types. When a type
    /// is formatted with a name that matches a mapped type, the mapped type
    /// name is formatted instead.
    fn mapped_type(&self, type_name: &TypeName) -> Option<&TypeName>;

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
    fn begin_file(&self, w: &mut impl Write, mode: FilesMode<&CrateName>) -> std::io::Result<()>;

    /// For generating import statements. This is called only in multi-file
    /// mode, after `begin_file` and before any another methods.
    fn write_imports(
        &self,
        writer: &mut impl Write,
        crate_name: &CrateName,
        imports: &ScopedCrateTypes<'_>,
    ) -> std::io::Result<()>;

    /// Implementors can use this function to write a footer for typeshared code.
    /// By default this does nothing.
    fn end_file(&self, w: &mut impl Write) -> std::io::Result<()>;

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

    /// Write a constant variable.
    /// Example of a constant variable:
    /// ```
    /// const ANSWER_TO_EVERYTHING: u32 = 42;
    /// ```
    fn write_const(&self, w: &mut impl Write, c: &RustConst) -> std::io::Result<()>;

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
