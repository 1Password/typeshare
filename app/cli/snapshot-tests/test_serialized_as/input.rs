#[typeshare(serialized_as = "String")]
pub struct ItemId {
    inner: i64,
}

/// Options that you could pick
#[typeshare(serialized_as = "String")]
pub enum Options {
    /// Affirmative Response
    Yes,
    No,
    Maybe,
    /// Sends a string along
    Cool(String),
}
