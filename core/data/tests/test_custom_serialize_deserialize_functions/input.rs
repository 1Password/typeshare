#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct Foo {
    pub time: time::OffsetDateTime,
    pub time2: time::OffsetDateTime,
    pub time3: time::OffsetDateTime,
    pub bytes: Vec<u8>,
    pub bytes2: Vec<u8>
}
