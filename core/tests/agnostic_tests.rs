use std::io::Write;
use typeshare_core::{
    language::{CrateTypes, Language, TypeScript},
    parser::{self, ParseError},
    rust_types::RustTypeParseError,
    ProcessInputError,
};
/// Parse and generate types for a single Rust input file.
pub fn process_input(
    input: &str,
    language: &mut dyn Language,
    imports: &CrateTypes,
    out: &mut dyn Write,
) -> Result<(), ProcessInputError> {
    let mut parsed_data = parser::parse(
        input,
        "default_name".into(),
        "file_name".into(),
        "file_path".into(),
        &[],
        false,
        None,
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

mod blocklisted_types {
    use std::collections::HashMap;

    use super::*;

    fn assert_type_is_blocklisted(ty: &str, blocklisted_type: &str) {
        let source = format!(
            r##"
    #[typeshare]
    #[serde(default, rename_all = "camelCase")]
    pub struct Foo {{
        pub bar: {ty},
    }}
    "##,
            ty = ty
        );

        let mut out: Vec<u8> = Vec::new();
        assert!(matches!(
            process_input(&source, &mut TypeScript::default(), &HashMap::new(), &mut out),
            Err(ProcessInputError::ParseError(
                ParseError::RustTypeParseError(RustTypeParseError::UnsupportedType(contents))
            )) if contents == vec![blocklisted_type.to_owned()]
        ));
    }

    #[test]
    fn test_i64_blocklisted_struct() {
        assert_type_is_blocklisted("i64", "i64");
    }

    #[test]
    fn test_u64_blocklisted_struct() {
        assert_type_is_blocklisted("u64", "u64");
    }

    #[test]
    fn test_isize_blocklisted_struct() {
        assert_type_is_blocklisted("isize", "isize");
    }

    #[test]
    fn test_usize_blocklisted_in_struct() {
        assert_type_is_blocklisted("usize", "usize");
    }

    #[test]
    fn test_optional_blocklisted_struct() {
        assert_type_is_blocklisted("Option<i64>", "i64");
    }

    #[test]
    fn test_vec_blocklisted_struct() {
        assert_type_is_blocklisted("Vec<i64>", "i64");
    }

    #[test]
    fn test_hashmap_blocklisted_struct() {
        assert_type_is_blocklisted("HashMap<String, i64>", "i64");
    }
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
