/// This is a comment.
/// Continued lovingly here
#[typeshare]
#[serde(rename_all = "camelCase")]
pub enum Colors {
    Red = 0,
    Blue = 1,
    /// Green is a cool color
    #[serde(rename = "green-like")]
    Green = 2,
}
