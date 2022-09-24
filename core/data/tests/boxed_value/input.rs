/// This is a comment.
#[typeshare]
#[serde(tag = "type", content = "content")]
pub enum Colors {
    Red,
    Blue,
    Green(Box<String>),
}
