use anyhow::anyhow;
use std::io::Write;
use typeshare_core::{
    context::{ParseContext, ParseFileContext},
    language::{CrateTypes, Language, TypeScript},
    parser::{self},
};

/// Parse and generate types for a single Rust input file.
pub fn process_input(
    input: &str,
    language: &mut dyn Language,
    imports: &CrateTypes,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let parse_context = ParseContext::default();

    let parsed_data = parser::parse(
        &parse_context,
        ParseFileContext {
            source_code: input.to_string(),
            crate_name: "default_name".into(),
            file_name: "file_name".into(),
            file_path: "file_path".into(),
        },
    )
    .map_err(|err| anyhow!("Failed to parse {err}"))?
    .unwrap();

    if !parsed_data.errors.is_empty() {
        return Err(anyhow!("{}", parsed_data.errors[0].error));
    }

    language.generate_types(out, imports, parsed_data)?;
    Ok(())
}

mod serde_attributes_on_enums {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn content_not_allowed_on_non_algebraic() {
        let source = r##"
    #[typeshare]
    #[serde(content = "bla")]
    pub enum Foo {
        Variant1,
        Variant2,
    }
    "##;

        let mut out: Vec<u8> = Vec::new();
        let err = process_input(
            source,
            &mut TypeScript::default(),
            &HashMap::new(),
            &mut out,
        )
        .unwrap_err();
        assert_eq!(
            err.to_string(),
            "The serde content attribute is not supported for non-algebraic enums: Foo, on line 2 and column 4"
        );
    }

    #[test]
    fn tag_not_allowed_on_non_algebraic() {
        let source = r##"
    #[typeshare]
    #[serde(tag = "bla")]
    pub enum Foo {
        Variant1,
        Variant2,
    }
    "##;

        let mut out: Vec<u8> = Vec::new();
        let err = process_input(
            source,
            &mut TypeScript::default(),
            &HashMap::new(),
            &mut out,
        )
        .unwrap_err();
        assert_eq!(
            err.to_string(),
            "The serde tag attribute is not supported for non-algebraic enums: Foo, on line 2 and column 4"
        );
    }

    #[test]
    fn both_not_allowed_on_non_algebraic() {
        let source = r##"
    #[typeshare]
    #[serde(tag = "bla", content = "bla2")]
    pub enum Foo {
        Variant1,
        Variant2,
    }
    "##;

        let mut out: Vec<u8> = Vec::new();
        let err = process_input(
            source,
            &mut TypeScript::default(),
            &HashMap::new(),
            &mut out,
        )
        .unwrap_err();
        assert_eq!(
            err.to_string(),
            "The serde tag attribute is not supported for non-algebraic enums: Foo, on line 2 and column 4"
        );
    }

    #[test]
    fn no_flatten() {
        let source = r##"
        #[typeshare]
        pub struct Foo {
            #[serde(flatten)]
            pub field1: HashMap<String, String>
        }
        "##;

        let mut out: Vec<u8> = Vec::new();
        let err = process_input(
            source,
            &mut TypeScript::default(),
            &HashMap::new(),
            &mut out,
        )
        .unwrap_err();
        assert_eq!(
            err.to_string(),
            "The serde flatten attribute is not currently supported, on line 4 and column 12"
        );
    }
}
