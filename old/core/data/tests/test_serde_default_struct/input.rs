#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct Foo {
    #[serde(default)]
    pub bar: bool,
}
