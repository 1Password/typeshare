use anyhow::anyhow;
use anyhow::Context;
use flexi_logger::DeferredNow;
use log::Record;
use once_cell::sync::Lazy;
use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Once,
};
use typeshare_core::{
    context::{ParseContext, ParseFileContext},
    language::{CrateName, Language},
    reconcile::reconcile_aliases,
};

static TESTS_FOLDER_PATH: Lazy<PathBuf> =
    Lazy::new(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/tests"));

static INIT: Once = Once::new();

fn init_log() {
    INIT.call_once(|| {
        flexi_logger::Logger::try_with_env()
            .unwrap()
            .format(
                |write: &mut dyn Write, _now: &mut DeferredNow, record: &Record<'_>| {
                    let file_name = record.file().unwrap_or_default();
                    let file_name = if file_name.len() > 15 {
                        let split = file_name.len() - 15;
                        &file_name[split..]
                    } else {
                        file_name
                    };
                    write!(
                        write,
                        "{file_name:>15}{:>5} - {}",
                        record.line().unwrap_or_default(),
                        record.args()
                    )
                },
            )
            .start()
            .unwrap();
    })
}

/// Reads the contents of the file at `path` into a string and returns it
///
/// The file will be created if it does not exist, as well as any
/// directories leading up to it.
fn load_file(path: impl AsRef<Path>) -> Result<String, anyhow::Error> {
    let path = path.as_ref();

    if path.file_name().is_some() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
    }

    let mut file = OpenOptions::new()
        .read(true)
        .open(path)
        .with_context(|| format!("failed to open file at path {}", path.to_string_lossy()))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).with_context(|| {
        format!(
            "failed to read from file at path {}",
            path.to_string_lossy()
        )
    })?;

    Ok(contents)
}

/// Performs a snapshot test for the given parameters
///
/// Provide a test name, the name of the output file to use for the test, and
/// a `Generator` instance appropriate for the test.
///
/// `input.rs` and `file_name` will both be created inside of the `test_name`
/// folder if they do not exist.
fn check(
    test_name: &str,
    file_name: impl AsRef<Path> + std::fmt::Debug,
    mut lang: Box<dyn Language>,
    target_os: &[&str],
) -> Result<(), anyhow::Error> {
    let _extension = file_name
        .as_ref()
        .extension()
        .expect("file name must have an extension");
    let expected_file_path = TESTS_FOLDER_PATH.join(test_name).join(&file_name);

    let rust_input = load_file(
        expected_file_path
            .with_file_name("input")
            .with_extension("rs"),
    )?;

    let mut typeshare_output: Vec<u8> = Vec::new();
    let parse_context = ParseContext {
        target_os: target_os.iter().map(ToString::to_string).collect(),
        ..Default::default()
    };

    let parsed_data = typeshare_core::parser::parse(
        &parse_context,
        ParseFileContext {
            source_code: rust_input,
            crate_name: "default_crate".into(),
            file_name: file_name.as_ref().to_string_lossy().to_string(),
            file_path: file_name.as_ref().into(),
        },
    )
    .map_err(|err| anyhow!("Parsing failed: {:?},  {err}", file_name))
    .inspect_err(|err| {
        eprintln!("Error: {err}");
    })?
    .unwrap();

    let all_crates: CrateName = String::new().into();

    let mut map = BTreeMap::from_iter([(all_crates.clone(), parsed_data)]);
    reconcile_aliases(&mut map);

    let parsed_data = map.remove(&all_crates).unwrap();

    if !parsed_data.errors.is_empty() {
        for error in &parsed_data.errors {
            eprintln!("Parsing failed: {error:?}");
        }
        panic!("Errors during parsing");
    }

    lang.generate_types(&mut typeshare_output, &HashMap::new(), parsed_data)?;

    let typeshare_output = String::from_utf8(typeshare_output)?;
    let expected = expect_test::expect_file![&expected_file_path];
    // Ensure that the unformatted typescript output matches what we expect
    expected.assert_eq(&typeshare_output);

    Ok(())
}

/// Makes a string literal representing the correct output filename for the
/// given ident
macro_rules! output_file_for_ident {
    (kotlin) => {
        "output.kt"
    };
    (scala) => {
        "output.scala"
    };
    (swift) => {
        "output.swift"
    };
    (typescript) => {
        "output.ts"
    };
    (go) => {
        "output.go"
    };
    (python) => {
        "output.py"
    };
}

/// Simplifies the construction of `Language` instances for each language.
///
/// Passing only the name of the language constructs an instance with default
/// parameters, while specific parameters can be provided within curly
/// brackets.
///
/// # Examples
///
/// Constructing a default instance for kotlin:
///
/// ```
/// let instance = language_instance!(kotlin);
/// ```
///
/// Constructing a kotlin instance with specific params:
///
/// ```
/// let instance = language_instance!(kotlin {
///     package: "com.agilebits.onepassword".to_string(),
///     module_name: String::new(),
/// });
/// ```
///
/// This expands to the following:
///
/// ```
/// let instance = crate::kotlin::language(crate::kotlin::Params {
///     package: "com.agilebits.onepassword".to_string(),
///     module_name: String::new(),
/// });
/// ```
macro_rules! language_instance {
    // Default kotlin
    (kotlin) => {
        language_instance!(kotlin {
            package: "com.agilebits.onepassword".to_string(),
            module_name: String::new(),
            type_mappings: Default::default(),
        })
    };

    // kotlin with configuration fields forwarded
    (kotlin {$($field:ident: $val:expr),* $(,)?}) => {
        #[allow(clippy::needless_update)]
        Box::new(typeshare_core::language::Kotlin {
            no_version_header: true,
            $($field: $val,)*
            ..Default::default()
        })
    };

	 // Default Python
	 (python) => {
        language_instance!(python { })
    };

    // python with configuration fields forwarded
    (python {$($field:ident: $val:expr),* $(,)?}) => {
        #[allow(clippy::needless_update)]
        Box::new(typeshare_core::language::Python {
            no_version_header: true,
            $($field: $val,)*
            ..Default::default()
        })
    };

    // Default scala
    (scala) => {
        language_instance!(scala {
            package: "com.agilebits.onepassword".to_string(),
            module_name: String::new(),
            type_mappings: Default::default(),
        })
    };

    // scala with configuration fields forwarded
    (scala {$($field:ident: $val:expr),* $(,)?}) => {
        #[allow(clippy::needless_update)]
        Box::new(typeshare_core::language::Scala {
            no_version_header: true,
            $($field: $val,)*
            ..Default::default()
        })
    };

    // Default swift
    (swift) => {
        language_instance!(swift { })
    };

    // swift with configuration fields forwarded
    (swift {$($field:ident: $val:expr),* $(,)?}) => {
        #[allow(clippy::needless_update)]
        Box::new(typeshare_core::language::Swift {
            no_version_header: true,
            $($field: $val,)*
            ..Default::default()
        })
    };

    // Default Typescript
    (typescript) => {
        language_instance!(typescript { })
    };

    // typescript with configuration fields forwarded
    (typescript {$($field:ident: $val:expr),* $(,)?}) => {
        #[allow(clippy::needless_update)]
        Box::new(typeshare_core::language::TypeScript {
            no_version_header: true,
            $($field: $val,)*
            ..Default::default()
        })
    };

     // Default Go
    (go) => {
        language_instance!(go { })
    };

     // Go with configuration fields forwarded
    (go {$($field:ident: $val:expr),* $(,)?}) => {
        #[allow(clippy::needless_update)]
        Box::new(typeshare_core::language::Go {
             package: "proto".to_string(),
             no_version_header: true,
             $($field: $val,)*
            ..Default::default()
        })
    };
}

macro_rules! target_os {
    (
        [$($target_os:literal),*]
    ) => {
        &[$($target_os),*]
    };

    () => {
        &[]
    };
}

/// This macro removes the boilerplate involved in creating typeshare snapshot
/// tests. Usage looks like:
///
/// ```
/// tests! {
///     generate_types: [kotlin, swift, typescript];
///     /// Comments work here too
///     some_other_test: [swift];
/// }
/// ```
///
/// Here we've declared a test named `generate_types` that has expectations for
/// kotlin, swift, and typescript code generation. This requires a folder named
/// `generate_types` to be present in `data/tests/`. The folder must contain:
///
/// * The input Rust source code (`input.rs`)
/// * The expected output for each language (`output.(kt|swift|ts)`)
///
/// We've also declared a test named `some_other_test` that only has an
/// expectation for the generated swift code. This test will not require
/// expectations for the other languages to be present.
///
/// If you need more control, the macro also supports the following syntax:
///
/// ```
/// tests! {
///    can_generate_algebraic_enum: [
///        swift {
///            prefix: "OP".to_string(),
///        },
///        kotlin {
///            package: "com.agilebits.onepassword".to_string(),
///            module_name: "colorsModule".to_string(),
///        },
///        typescript
///    ];
/// }
/// ```
///
/// Here we've specified a test with expectations for all three languages, and
/// we've additionally passed some custom parameters to be used in the swift
/// and kotlin tests. These parameters are used as the fields for the
/// languages' `Params` struct.
///
/// This macro outputs the following structure:
///
/// ```
/// mod $test {
///     #[test]
///     fn kotlin() {
///         // ...
///     }
///
///     #[test]
///     fn swift() {
///         // ...
///     }
///
///     #[test]
///     fn typescript() {
///         // ...
///     }
/// }
/// ```
macro_rules! tests {
    // The initial `$(#[$outer:meta])*` here captures comments so they can be used
    // inside the macro; we don't do anything with them, though
    //
    // The `$(,)?` towards the end makes trailing commas valid
    ($(
        $(#[$outer:meta])*
        $test:ident : [
            $(
                $language:ident $({
                    $($lang_config:tt)*
                })?
            ),+
            $(,)?
        ] $(target_os: $target_os:tt)?;
    )*) => {$(
        mod $test {
            use super::check;

            const TEST_NAME: &str = stringify!($test);
            const TARGET_OS: &[&str] = target_os!($($target_os)?);

            $(
                #[test]
                fn $language() -> Result<(), anyhow::Error> {
                    crate::init_log();
                    check(
                        TEST_NAME,
                        output_file_for_ident!($language),
                        language_instance!($language $({ $($lang_config)* })?),
                        TARGET_OS
                    )
                }
            )+
        }
    )*};
}

static KOTLIN_MAPPINGS: Lazy<HashMap<String, String>> = Lazy::new(|| {
    [("Url", "String"), ("DateTime", "String")]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
});

static SCALA_MAPPINGS: Lazy<HashMap<String, String>> = Lazy::new(|| {
    [("Url", "String"), ("DateTime", "String")]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
});

static SWIFT_MAPPINGS: Lazy<HashMap<String, String>> = Lazy::new(|| {
    [("Url", "String"), ("DateTime", "Date")]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
});

static TYPESCRIPT_MAPPINGS: Lazy<HashMap<String, String>> = Lazy::new(|| {
    [
        ("Url", "string"),
        ("DateTime", "string"),
        ("Vec<u8>", "Uint8Array"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
});

static GO_MAPPINGS: Lazy<HashMap<String, String>> = Lazy::new(|| {
    [
        ("Url", "string"),
        ("DateTime", "string"),
        ("Vec<u8>", "[]byte"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
});

static PYTHON_MAPPINGS: Lazy<HashMap<String, String>> = Lazy::new(|| {
    [
        ("Url", "AnyUrl"),
        ("DateTime", "datetime"),
        ("Vec<u8>", "bytes"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
});

tests! {
    /// Enums
    can_generate_algebraic_enum: [
        swift {
            prefix: "OP".to_string(),
        },
        kotlin {
            package: "com.agilebits.onepassword".to_string(),
            module_name: "colorsModule".to_string(),
        },
        scala {
            package: "com.agilebits.onepassword".to_string(),
            module_name: "colorsModule".to_string(),
        },
        typescript,
        go,
        python
    ];
    can_generate_generic_enum: [
        swift {
            prefix: "Core".into(),
        },
        kotlin,
        scala,
        typescript
    ];
    can_generate_generic_struct: [
        swift {
            prefix: "Core".into(),
            codablevoid_constraints: vec!["Equatable".into()]
        },
        kotlin,
        scala,
        typescript,
    ];
    can_generate_generic_type_alias: [
        swift {
            prefix: "Core".into()
        },
        kotlin,
        scala,
        typescript
    ];
    can_generate_const: [typescript, go, python];
    can_generate_slice_of_user_type: [swift, kotlin, scala, typescript, go, python];
    can_generate_readonly_fields: [
        typescript
    ];
    can_generate_simple_enum: [
        swift {
            prefix: "TypeShare".to_string(),
        },
        kotlin,
        scala,
        typescript,
        go,
        python
    ];
    can_generate_bare_string_enum: [swift, kotlin, scala, typescript, go, python ];
    can_generate_double_option_pattern: [
        typescript
    ];
    can_recognize_types_inside_modules: [
        swift, kotlin, scala, typescript, go, python
    ];
    test_simple_enum_case_name_support: [swift, kotlin, scala, typescript, go, python ];
    test_algebraic_enum_case_name_support: [
        swift {
            prefix: "OP".to_string(),
        },
        kotlin {
            package: "com.agilebits.onepassword".to_string(),
            module_name: "colorModule".to_string(),
        },
        scala {
            package: "com.agilebits.onepassword".to_string(),
            module_name: "colorModule".to_string(),
        },
        typescript,
        go,
        python
    ];
    can_apply_prefix_correctly: [ swift { prefix: "OP".to_string(), }, kotlin { prefix: "OP".to_string(), }, scala,  typescript, go, python ];
    can_generate_empty_algebraic_enum: [ swift { prefix: "OP".to_string(), }, kotlin { prefix: "OP".to_string(), }, scala,  typescript, go, python ];
    can_generate_algebraic_enum_with_skipped_variants: [swift, kotlin, scala,  typescript, go, python];
    can_generate_struct_with_skipped_fields: [swift, kotlin, scala,  typescript, go, python];
    enum_is_properly_named_with_serde_overrides: [swift, kotlin, scala,  typescript, go, python];
    can_handle_quote_in_serde_rename: [swift, kotlin, scala,  typescript, go, python];
    can_handle_anonymous_struct: [swift, kotlin, scala,  typescript, go, python];
    test_generate_char: [swift, kotlin, scala, typescript, go, python];
    anonymous_struct_with_rename: [
        swift {
            prefix: "Core".to_string(),
        },
        kotlin,
        scala,
        typescript,
        go,
        python
    ];
    can_override_types: [swift, kotlin, scala, typescript, go];
    can_override_disallowed_types: [swift, kotlin, scala, typescript, go];

    /// Structs
    can_generate_simple_struct_with_a_comment: [kotlin, swift, typescript, scala, go, python];
    generate_types: [kotlin, swift, typescript, scala,  go, python];
    can_handle_serde_rename: [
        swift {
            prefix: "TypeShareX_".to_string(),
        },
        kotlin,
        scala,
        typescript,
        go,
        python
    ];
    // TODO: kotlin and typescript don't appear to support this yet
    generates_empty_structs_and_initializers: [swift, kotlin, scala, typescript, go,python];
    test_default_decorators: [swift { default_decorators: vec!["Sendable".into(), "Identifiable".into()]}];
    test_default_generic_constraints: [swift { default_generic_constraints: typeshare_core::language::GenericConstraints::from_config(vec!["Sendable".into(), "Identifiable".into()]) }];
    test_i54_u53_type: [swift, kotlin, scala,  typescript, go, python];
    test_serde_default_struct: [swift, kotlin, scala,  typescript, go, python];
    test_serde_iso8601: [
        swift {
            prefix: String::new(),
            type_mappings: super::SWIFT_MAPPINGS.clone(),
        },
        kotlin {
            package: "com.agilebits.onepassword".to_string(),
            module_name: "colorModule".to_string(),
            type_mappings: super::KOTLIN_MAPPINGS.clone()
        },
        scala {
            package: "com.agilebits.onepassword".to_string(),
            module_name: "colorModule".to_string(),
            type_mappings: super::KOTLIN_MAPPINGS.clone()
        },
        typescript {
            type_mappings: super::TYPESCRIPT_MAPPINGS.clone(),
        },
         go {
            type_mappings: super::GO_MAPPINGS.clone(),
        },
        python {
            type_mappings: super::PYTHON_MAPPINGS.clone(),
        }
    ];
    test_serde_url: [
        swift {
            prefix: String::new(),
            type_mappings: super::SWIFT_MAPPINGS.clone(),
        },
        kotlin {
            package: "com.agilebits.onepassword".to_string(),
            module_name: "colorModule".to_string(),
            type_mappings: super::KOTLIN_MAPPINGS.clone()
        },
        scala {
            package: "com.agilebits.onepassword".to_string(),
            module_name: "colorModule".to_string(),
            type_mappings: super::SCALA_MAPPINGS.clone()
        },
        typescript {
            type_mappings: super::TYPESCRIPT_MAPPINGS.clone(),
        },
        go {
            type_mappings: super::GO_MAPPINGS.clone(),
            uppercase_acronyms: vec!["URL".to_string()],
        },
        python{
            type_mappings: super::PYTHON_MAPPINGS.clone()
        }
    ];
    test_type_alias: [ swift { prefix: "OP".to_string(), }, kotlin, scala,  typescript, go, python ];
    test_optional_type_alias: [swift, kotlin, scala, typescript, go, python];
    test_serialized_as: [ swift { prefix: "OP".to_string(), }, kotlin, scala,  typescript, go, python ];
    test_serialized_as_tuple: [
        swift {
            prefix: "OP".to_string(),
        },
        kotlin,
        scala,
        typescript,
        go {
            uppercase_acronyms: vec!["ID".to_string()],
        },
        python
    ];
    can_handle_serde_rename_all: [swift, kotlin, scala,  typescript, go,python];
    can_handle_serde_rename_on_top_level: [swift { prefix: "OP".to_string(), }, kotlin, scala,  typescript, go, python];
    can_generate_unit_structs: [swift, kotlin, scala, typescript, go, python];
    kebab_case_rename: [swift, kotlin, scala,  typescript, go, python];

    /// Globals get topologically sorted
    orders_types: [swift, kotlin, go, python];

    /// Other
    use_correct_integer_types: [swift, kotlin, scala,  typescript, go, python];
    // Only swift supports generating types with keywords
    generate_types_with_keywords: [swift];
    // TODO: how is this different from generates_empty_structs_and_initializers?
    use_correct_decoded_variable_name: [swift, kotlin, scala,  typescript, go, python];
    can_handle_unit_type: [swift { codablevoid_constraints: vec!["Equatable".into()]} , kotlin, scala,  typescript, go, python];

    //3 tests for adding decorators to enums and structs
    const_enum_decorator: [ swift{ prefix: "OP".to_string(), } ];
    algebraic_enum_decorator: [ swift{ prefix: "OP".to_string(), } ];
    struct_decorator: [ kotlin, swift{ prefix: "OP".to_string(), } ];
    serialize_field_as: [kotlin, swift, typescript, scala,  go, python];
    serialize_type_alias: [kotlin, swift, typescript, scala,  go, python];
    serialize_anonymous_field_as: [kotlin, swift, typescript, scala, go, python];
    smart_pointers: [kotlin, swift, typescript, scala, go, python];
    recursive_enum_decorator: [kotlin, swift, typescript, scala, go, python];

    uppercase_go_acronyms: [
        go {
            uppercase_acronyms: vec!["ID".to_string(), "url".to_string()],
        },
    ];
    resolves_qualified_type: [
        swift {
            prefix: "Core".into()
        },
        typescript,
        kotlin,
        scala,
        go,
        python
    ];
    can_generate_anonymous_struct_with_skipped_fields: [swift, kotlin, scala, typescript, go, python];
    generic_struct_with_constraints_and_decorators: [swift { codablevoid_constraints: vec!["Equatable".into()] }];
    excluded_by_target_os: [ swift, kotlin, scala, typescript, go,python ] target_os: ["android", "macos"];
    // excluded_by_target_os_full_module: [swift] target_os: "ios";
    serde_rename_references: [ swift, kotlin, scala, typescript, go ];
    test_custom_serialize_deserialize_functions: [    go
    {
        type_mappings: super::GO_MAPPINGS.clone(),
    },
    python
    {
        type_mappings: super::PYTHON_MAPPINGS.clone(),
    },
    typescript
    {
        type_mappings: super::TYPESCRIPT_MAPPINGS.clone(),
    }
    ];
    no_mangle: [swift, kotlin, scala, typescript, go];
}
