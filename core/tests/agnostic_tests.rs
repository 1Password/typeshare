use typeshare_core::{language::TypeScript, parser::ParseError, process_input, ProcessInputError};

mod serde_attributes_on_enums {
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
            process_input(source, &mut TypeScript::default(), &mut out).unwrap_err(),
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
            process_input(source, &mut TypeScript::default(), &mut out).unwrap_err(),
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
            process_input(source, &mut TypeScript::default(), &mut out).unwrap_err(),
            ProcessInputError::ParseError(ParseError::SerdeTagNotAllowed { enum_ident }) if enum_ident == "Foo"
        ));
    }
}
