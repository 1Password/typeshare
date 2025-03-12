#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct Foo {
    pub time: OffsetDateTime,
    pub time2: OffsetDateTime
}
