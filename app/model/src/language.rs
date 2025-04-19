use std::{borrow::Cow, fmt::Debug, io::Write, path::Path};

use anyhow::Context;
use itertools::Itertools;

use crate::parsed_data::{
    CrateName, Id, RustConst, RustEnum, RustEnumVariant, RustStruct, RustType, RustTypeAlias,
    SpecialRustType, TypeName,
};

/// If we're in multifile mode, this enum contains the crate name for the
/// specific file
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum FilesMode<T> {
    Single,
    Multi(T),
    // We've had requests for java support, which means we'll need a
    // 1-file-per-type mode
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

/**
*The* trait you need to implement in order to have your own implementation of
typeshare. The whole world revolves around this trait.

In general, to implement this correctly, you *must* implement:

- `new_from_config`, which instantiates your `Language` struct from
  configuration which was read from a config file or the command line
- `output_filename_for_crate`, which (in multi-file mode) produces a file
  name from a crate name. All of the typeshared types from that crate will
  be written to that file.
- The `write_*` methods, which output the actual type definitions. These
  methods *should* call `format_type` to format the actual types contained
  in the type definitions, which will in turn dispatch to the relevant
  `format_*` method, depending on what kind of type it is.
- The `format_special_type` method, which outputs things like integer types,
  arrays, and other builtin or primitive types. This method is only ever called
  by `format_type`, which is only called if you choose to call it in your
  `write_*` implementations.

Additionally, you must provide a `Config` associated type, which must implement
`Serialize + Deserialize`. This type will be used to load configuration from
a config file and from the command line arguments for your language, which will
be passed to `new_from_config`. This type should provide defaults for *all* of
its fields; it should always tolerate being loaded from an empty config file.
When *serializing*, this type should always output all of its fields, even if
they're defaulted.

It's also very common to implement:

- `mapped_type`, to define certain types as having specialied handling in your
  lanugage.
- `begin_file`, `end_file`, and `write_additional_files`, to add additional
  per-file or per-directory content to your output.

If your language spells type names in an unusual way (here, defined as the C++
descended convention, where a type might be spelled `Foo<Bar, Baz<Zed>>`),
you'll want to implement the `format_*` methods.

Other methods can be specialized as needed.

# Typeshare execution flow.

This is the detailed flow of how the `Language` trait is actually used by
typeshare. It includes references to all of the methods that are called, and
in what order. For these examples, we're assuming a hypothetical implementation
for Kotlin, which means that there must be `impl Language<'_> for Kotlin`
somewhere.

1. The language's config is loaded from the config file and command line
arguments:

```ignore
let config = Kotlin::Config::deserialize(config_file)?;
```

2. The language is loaded from the config file via `new_from_config`. This is
where the implementation has the opportunity to report any configuration errors
that weren't detected during deserialization.

```ignore
let language = Kotlin::new_from_config(config)?;
```

3. If we're in multi-file mode, we call `output_filename_for_crate` for each rust
crate being typeshared to determine the _filename_ for the output file that
will contain that crate's types.

```ignore
let files = crate_names
    .iter()
    .map(|crate_name| {
        let filename = language.output_filename_for_crate(crate_name);
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
// Only in multi-file mode
language.write_imports(&mut file, crate_name, computed_imports)
```

6. For EACE typeshared item in being typeshared, we call `write_enum`,
`write_struct`, `write_type_alias`, or `write_const`, as appropriate.

```ignore
language.write_struct(&mut file, parsed_struct);
language.write_enum(&mut file, parsed_enum);
```

6a. In your implementations of these methods, we recommend that you call
`format_type` for the fields of these types. `format_type` will in turn call
`format_simple_type`, `format_generic_type`, or `format_special_type`, as
appropriate; usually it is only necessary for you to implmenent
`format_special_type` yourself, and use the default implementations for the
others. The `format_*` methods will otherwise never be called by typeshare.

6b. If your language doesn't natively support data-containing enums, we
recommand that you call `write_types_for_anonymous_structs` in your
implementation of `write_enum`; this will call `write_struct` for each variant
of the enum.

7. After all the types are written, we call `end_file`, with the same
arguments that were passed to `begin_file`.

```ignore
language.end_file(&mut file, mode)
```

8. In multi-file mode only, after ALL files are written, we call
`write_additional_files` with the output directory. This gives the language an
opportunity to create any files resembling `mod.rs` or `index.js` as it might
require.

```ignore
// Only in multi-file mode
language.write_additional_files(&output_directory, generated_files.iter())
```

NOTE: at this stage, multi-file output is still work-in-progress, as the
algorithms that compute import sets are being rewritten. The API presented
here is stable, but output might be buggy while issues with import detection
are resolved.
*/
pub trait Language<'config>: Sized + Sync + Debug {
    /**
    The configuration for this language. This configuration will be loaded
    from a config file and, where possible, from the command line, via
    `serde`.

    It is important that this type include `#[serde(default)]` or something
    equivelent, so that a config can be loaded with default setting even
    if this language isn't present in the config file.

    The `serialize` implementation for this type should NOT skip keys, if
    possible.
    */
    type Config: serde::Deserialize<'config> + serde::Serialize;

    /**
    The lowercase conventional name for this language. This should be a
    single identifier. It will be used as a prefix for various things;
    for instance, it will identify this language in the config file, and
    be used as a prefix when generating CLI parameters
    */
    const NAME: &'static str;

    /// Create an instance of this language from the loaded configuration.
    fn new_from_config(config: Self::Config) -> anyhow::Result<Self>;

    /**
    Most languages provide manual overrides for specific types. When a type
    is formatted with a name that matches a mapped type, the mapped type
    name is formatted instead.

    By default this returns `None` for all types.
    */
    fn mapped_type(&self, type_name: &TypeName) -> Option<Cow<'_, str>> {
        let _ = type_name;
        None
    }

    /**
    In multi-file mode, typeshare will output one separate file with this
    name for each crate in the input set. These file names should have the
    appropriate naming convention and extension for this language.

    This method isn't used in single-file mode.
    */
    fn output_filename_for_crate(&self, crate_name: &CrateName) -> String;

    /**
    Convert a Rust type into a type from this language. By default this
    calls `format_simple_type`, `format_generic_type`, or
    `format_special_type`, depending on the type. There should only rarely
    be a need to specialize this.

    This method should be called by the `write_*` methods to write the types
    contained by type definitions.

    The `generic_context` is the set of generic types being provided by
    the enclosing type definition; this allows languages that do type
    renaming to be able to distinguish concrete type names (like `int`)
    from generic type names (like `T`)
    */
    fn format_type(&self, ty: &RustType, generic_context: &[TypeName]) -> anyhow::Result<String> {
        match ty {
            RustType::Simple { id } => self.format_simple_type(id, generic_context),
            RustType::Generic { id, parameters } => {
                self.format_generic_type(id, parameters.as_slice(), generic_context)
            }
            RustType::Special(special) => self.format_special_type(special, generic_context),
        }
    }

    /**
    Format a simple type with no generic parameters.

    By default, this first checks `self.mapped_type` to see if there's an
    alternative way this type should be formatted, and otherwise prints the
    `base` verbatim.

    The `generic_context` is the set of generic types being provided by
    the enclosing type definition; this allows languages that do type
    renaming to be able to distinguish concrete type names (like `int`)
    from generic type names (like `T`).
    */
    fn format_simple_type(
        &self,
        base: &TypeName,
        generic_context: &[TypeName],
    ) -> anyhow::Result<String> {
        let _ = generic_context;
        Ok(match self.mapped_type(base) {
            Some(mapped) => mapped.to_string(),
            None => base.to_string(),
        })
    }

    /**
    Format a generic type that takes in generic arguments, which
    may be recursive.

    By default, this creates a composite type name by appending
    `self.format_simple_type` and `self.format_generic_parameters`. With
    their default implementations, this will print `name<parameters>`,
    which is a common syntax used by many languages for generics.

    The `generic_context` is the set of generic types being provided by
    the enclosing type definition; this allows languages that do type
    renaming to be able to distinguish concrete type names (like `int`)
    from generic type names (like `T`).
    */
    fn format_generic_type(
        &self,
        base: &TypeName,
        parameters: &[RustType],
        generic_context: &[TypeName],
    ) -> anyhow::Result<String> {
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

    /**
    Format generic parameters into a syntax used by this language. By
    default, this returns `<A, B, C, ...>`, since that's a common syntax
    used by most languages.

    This method is only used when `format_generic_type` calls it.

    The `generic_context` is the set of generic types being provided by
    the enclosing type definition; this allows languages that do type
    renaming to be able to distinguish concrete type names (like `int`)
    from generic type names (like `T`).
    */
    fn format_generic_parameters(
        &self,
        parameters: &[RustType],
        generic_context: &[TypeName],
    ) -> anyhow::Result<String> {
        parameters
            .iter()
            .map(|ty| self.format_type(ty, generic_context))
            .process_results(|mut formatted| format!("<{}>", formatted.join(", ")))
    }

    /**
    Format a special type. This will handle things like arrays, primitives,
    options, and so on. Every lanugage has different spellings for these types,
    so this is one of the key methods that a language implementation needs to
    deal with.
    */
    fn format_special_type(
        &self,
        special_ty: &SpecialRustType,
        generic_context: &[TypeName],
    ) -> anyhow::Result<String>;

    /**
    Write a header for typeshared code. This is called unconditionally
    at the start of the output file (or at the start of all files, if in
    multi-file mode).

    By default this does nothing.
    */
    fn begin_file(&self, w: &mut impl Write, mode: FilesMode<&CrateName>) -> anyhow::Result<()> {
        let _ = (w, mode);
        Ok(())
    }

    /**
    For generating import statements. This is called only in multi-file
    mode, after `begin_file` and before any other writes.

    `imports` includes an ordered list of type names that typeshare
    believes are being imported by this file, grouped by the crates they
    come from. `typeshare` guarantees that these will be passed in some stable
    order, so that your output remains consistent.

    NOTE: Currently this is bugged and doesn't receive correct imports.
    This will be fixed in a future release.
    */
    fn write_imports<'a, Crates, Types>(
        &self,
        writer: &mut impl Write,
        crate_name: &CrateName,
        imports: Crates,
    ) -> anyhow::Result<()>
    where
        Crates: IntoIterator<Item = (&'a CrateName, Types)>,
        Types: IntoIterator<Item = &'a TypeName>;

    /**
    Write a header for typeshared code. This is called unconditionally
    at the end of the output file (or at the end of all files, if in
    multi-file mode).

    By default this does nothing.
    */
    fn end_file(&self, w: &mut impl Write, mode: FilesMode<&CrateName>) -> anyhow::Result<()> {
        let _ = (w, mode);
        Ok(())
    }

    /**
    Write a type alias definition.

    Example of a type alias:
    ```
    type MyTypeAlias = String;
    ```

    Generally this method will call `self.format_type` to produce the
    aliased type name in the output definition.
    */
    fn write_type_alias(&self, w: &mut impl Write, t: &RustTypeAlias) -> anyhow::Result<()>;

    /**
    Write a struct definition.

    Example of a struct:
    ```ignore
    #[typeshare]
    #[derive(Serialize, Deserialize)]
    struct Foo {
        bar: String
    }
    ```

    Generally this method will call `self.format_type` to produce the types
    of the individual fields.
    */
    fn write_struct(&self, w: &mut impl Write, rs: &RustStruct) -> anyhow::Result<()>;

    /**
    Write an enum definition.

    Example of an enum:
    ```ignore
    #[typeshare]
    #[derive(Serialize, Deserialize)]
    #[serde(tag = "type", content = "content")]
    enum Foo {
        Fizz,
        Buzz { yep_this_works: bool }
    }
    ```

    Generally this will call `self.format_type` to produce the types of
    the individual fields. If this enum is an algebraic sum type, and this
    language doesn't really support those, it should consider calling
    `write_struct_types_for_enum_variants` to produce struct types matching
    those variants, which can be used for this language's abstraction for
    data like this.
    */
    fn write_enum(&self, w: &mut impl Write, e: &RustEnum) -> anyhow::Result<()>;

    /**
    Write a constant variable.

    Example of a constant variable:
    ```
    const ANSWER_TO_EVERYTHING: u32 = 42;
    ```

    If necessary, generally this will call `self.format_type` to produce
    the type of this constant (though some languages are allowed to omit
    it).
    */
    fn write_const(&self, w: &mut impl Write, c: &RustConst) -> anyhow::Result<()>;

    /**
    Write out named types to represent anonymous struct enum variants.

    Take the following enum as an example:

    ```
    enum AlgebraicEnum {
        AnonymousStruct {
            field: String,
            another_field: bool,
        },

        Variant2 {
            field: i32,
        }
    }
    ```

    This function will write out a pair of struct types resembling:

    ```
    struct AnonymousStruct {
        field: String,
        another_field: bool,
    }

    struct Variant2 {
        field: i32,
    }
    ```

    Except that it will use `make_struct_name` to compute the names of these
    structs based on the names of the variants.

    This method isn't called by default; it is instead provided as a helper
    for your implementation of `write_enum`, since many languages don't have
    a specific notion of an algebraic sum type, and have to emulate it with
    subclasses, tagged unions, or something similar.
    */
    fn write_struct_types_for_enum_variants(
        &self,
        w: &mut impl Write,
        e: &RustEnum,
        make_struct_name: &impl Fn(&TypeName) -> String,
    ) -> anyhow::Result<()> {
        let variants = match e {
            RustEnum::Unit { .. } => return Ok(()),
            RustEnum::Algebraic { variants, .. } => variants.iter().filter_map(|v| match v {
                RustEnumVariant::AnonymousStruct { fields, shared } => Some((fields, shared)),
                _ => None,
            }),
        };

        for (fields, variant) in variants {
            let struct_name = make_struct_name(&variant.id.original);

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
                        renamed: TypeName::new_string(struct_name),
                    },
                    fields: fields.clone(),
                    generic_types,
                    comments: vec![format!(
                        "Generated type representing the anonymous struct \
                        variant `{}` of the `{}` Rust enum",
                        &variant.id.original,
                        &e.shared().id.original,
                    )],
                    decorators: e.shared().decorators.clone(),
                },
            )
            .with_context(|| {
                format!(
                    "failed to write struct type for the \
                    `{}` variant of the `{}` enum",
                    variant.id.original,
                    e.shared().id.original
                )
            })?;
        }

        Ok(())
    }

    /**
    If a type with this name appears in a type definition, it will be
    unconditionally excluded from cross-file import analysis. Usually this will
    be the types in `mapped_types`, since those are types with special behavior
    (for instance, a datetime date provided as a standard type by your
    langauge).

    This is mostly a performance optimization. By default it returns `false`
    for all types.
    */
    fn exclude_from_import_analysis(&self, name: &TypeName) -> bool {
        let _ = name;
        false
    }

    /**
    In multi-file mode, this method is called after all of the individual
    typeshare files are completely generated. Use it to generate any
    additional files your language might need in this directory to
    function correctly, such as a `mod.rs`, `__init__.py`, `index.js`, or
    anything else like that.

    It passed a list of crate names, for each crate that was typeshared, and
    the associated file paths, indicating all of the files that were generated
    by typeshare.

    By default, this does nothing.
    */
    fn write_additional_files<'a>(
        &self,
        output_folder: &Path,
        output_files: impl IntoIterator<Item = (&'a CrateName, &'a Path)>,
    ) -> anyhow::Result<()> {
        let _ = (output_folder, output_files);
        Ok(())
    }
}
