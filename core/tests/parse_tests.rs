use typeshare_core::parsed_types::Source;
use typeshare_core::rust_parser;

#[test]
fn basic_struct() {
    let source = r##"
    #[typeshare]
    pub struct Foo {
        pub bar: String,
    }
    "##;

    let parsed_data = rust_parser::parse(source, Source::default()).unwrap();
    println!("{:#?}", parsed_data);
}
#[test]
fn basic_struct_rename_all() {
    let source = r##"
    #[typeshare]
    #[serde(rename_all = "camelCase")]
    pub struct Foo {
        pub bar_foo: String,
        #[serde(rename = "nooooo")]
        pub bar: String,
    }
    "##;

    let parsed_data = rust_parser::parse(source, Source::default()).unwrap();
    println!("{:#?}", parsed_data);
}
#[test]
fn type_test() {
    let source = r##"
    #[typeshare]
    pub type Foo = String;
    #[typeshare(serialized_as = "i32")]
    pub type Bar = String;
    "##;

    let parsed_data = rust_parser::parse(source, Source::default()).unwrap();
    println!("{:#?}", parsed_data);
}

#[test]
fn enum_tests() {
    let source = r##"
    #[typeshare]
    pub enum Foo {
        Bar,
        Baz,
    }
    #[typeshare]
    #[serde(tag = "type", content = "value")]
    #[serde(rename_all = "camelCase")]
    pub enum Foo2 {
        Bar(String),
        #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
        BazFoo { bar: String },
    }
    "##;

    let parsed_data = rust_parser::parse(source, Source::default()).unwrap();
    println!("{:#?}", parsed_data);
}
#[test]
#[should_panic]
fn bad_enum_tests() {
    let source = r##"
    #[typeshare]
    #[serde(tag = "type", content = "value")]
    pub enum Foo {
        Bar,
        Baz,
    }
    #[typeshare]
    pub enum Foo2 {
        Bar(String),
        #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
        BazFoo { bar: String },
    }
    "##;

    let parsed_data = rust_parser::parse(source, Source::default()).unwrap();
    println!("{:#?}", parsed_data);
}
#[test]
fn typescript_test() {
    let source = r##"
    #[typeshare]
    pub struct Foo {
        #[typeshare(lang = typescript(type = "bigint"))]
        pub big_number: i64,
    }
    "##;

    let parsed_data = rust_parser::parse(source, Source::default()).unwrap();
    println!("{:#?}", parsed_data);
}
#[test]
fn skip_test() {
    let source = r##"
    #[typeshare]
    pub struct Foo {
        #[typeshare(skip)]
        pub big_number: i64,
    }
    "##;

    let parsed_data = rust_parser::parse(source, Source::default()).unwrap();
    println!("{:#?}", parsed_data);
}
