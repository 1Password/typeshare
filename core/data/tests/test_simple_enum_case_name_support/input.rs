/// This is a comment.
#[typeshare]
#[serde(rename_all = "camelCase")]
pub enum Colors {
    Red,
    #[serde(rename = "blue-ish")]
    Blue,
    #[serde(rename = "Green")]
    Green,
}
