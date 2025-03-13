//! Test references to a type that has been renamed via serde(rename)
//!

#[derive(Serialize)]
#[serde(rename = "SomethingFoo")]
#[typeshare]
pub enum Foo {
    A,
}

#[derive(Serialize)]
#[typeshare]
#[serde(tag = "type", content = "value")]
pub enum Parent {
    B(Foo),
}

#[derive(Serialize)]
#[typeshare]
pub struct Test {
    field1: Foo,
    field2: Option<Foo>,
}

#[derive(Serialize)]
#[typeshare]
pub type AliasTest = Vec<Foo>;
