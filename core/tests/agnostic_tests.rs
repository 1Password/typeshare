use std::io::Write;
use typeshare_core::{
    context::{ParseContext, ParseFileContext},
    language::{CrateTypes, Language, TypeScript},
    parser::{self, ParseError},
    ProcessInputError,
};
/// Parse and generate types for a single Rust input file.
pub fn process_input(
    input: &str,
    language: &mut dyn Language,
    imports: &CrateTypes,
    out: &mut dyn Write,
) -> Result<(), ProcessInputError> {
    let parse_context = ParseContext::default();

    let mut parsed_data = parser::parse(
        &parse_context,
        ParseFileContext {
            source_code: input.to_string(),
            crate_name: "default_name".into(),
            file_name: "file_name".into(),
            file_path: "file_path".into(),
        },
    )?
    .unwrap();

    if !parsed_data.errors.is_empty() {
        return Err(ProcessInputError::ParseError(
            parsed_data.errors.remove(0).error,
        ));
    }

    language.generate_types(out, imports, parsed_data)?;
    Ok(())
}

mod serde_attributes_on_enums {
    use std::collections::HashMap;

    use super::*;

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
        assert!(matches!(
            process_input(source, &mut TypeScript::default(), &HashMap::new(), &mut out).unwrap_err(),
            ProcessInputError::ParseError(ParseError::SerdeContentNotAllowed { enum_ident }) if enum_ident == "Foo"
        ));
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
        assert!(matches!(
            process_input(source, &mut TypeScript::default(), &HashMap::new(), &mut out).unwrap_err(),
            ProcessInputError::ParseError(ParseError::SerdeTagNotAllowed { enum_ident }) if enum_ident == "Foo"
        ));
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
        assert!(matches!(
            process_input(source, &mut TypeScript::default(), &HashMap::new(), &mut out).unwrap_err(),
            ProcessInputError::ParseError(ParseError::SerdeTagNotAllowed { enum_ident }) if enum_ident == "Foo"
        ));
    }
}
