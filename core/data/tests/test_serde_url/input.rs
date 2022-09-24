#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct Foo {
    pub url: url::Url,
}
