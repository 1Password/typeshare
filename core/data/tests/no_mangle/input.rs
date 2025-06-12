#[typeshare]
#[serde(rename_all = "camelCase")]
#[unsafe(no_mangle)]
pub struct Test {
    pub field_1: String,
    pub field_2: u32,
}
