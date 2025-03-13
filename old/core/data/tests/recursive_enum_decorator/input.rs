#[typeshare]
#[serde(tag = "type", content = "content", rename_all = "camelCase")]
pub enum Options {
    Red(bool),
    Banana(String),
    Vermont(Options),
}

#[typeshare]
#[serde(tag = "type", content = "content", rename_all = "camelCase")]
pub enum MoreOptions {
    News(bool),
    Exactly { config: String },
    Built { top: MoreOptions },
}
