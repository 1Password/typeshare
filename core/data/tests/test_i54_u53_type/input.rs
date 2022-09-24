#[typeshare]
#[serde(default, rename_all = "camelCase")]
pub struct Foo {
    pub a: I54,
    pub b: U53,
}
