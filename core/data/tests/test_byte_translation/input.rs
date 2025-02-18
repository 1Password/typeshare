#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct Foo {
    pub bytes: Vec<u8>,
}
