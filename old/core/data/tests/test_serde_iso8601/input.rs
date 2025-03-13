#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct Foo {
    pub time: DateTime<Utc>,
}
