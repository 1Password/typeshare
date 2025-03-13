#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct Foo {
    pub this_is_bits: Vec<u8>,
    pub this_is_redundant: Vec<u8>
}
