#[typeshare]
pub struct AccountId(String);

#[typeshare]
pub struct Foo {
    pub id: String,
    pub id_with_suffix: String,
    pub prefix_with_id: String,
    pub identity: String,
    pub lowercase_input_url: String,
    pub uppercase_type: AccountId,
}

#[typeshare]
#[serde(tag = "type", content = "content")]
pub enum Bar {
    Id(String),
}
