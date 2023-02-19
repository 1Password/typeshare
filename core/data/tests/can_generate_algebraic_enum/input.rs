/// Struct comment
#[typeshare]
pub struct ItemDetailsFieldValue {}

/// Enum comment
#[typeshare]
#[serde(tag = "type", content = "content")]
pub enum AdvancedColors {
    /// This is a case comment
    String(String),
    Number(i32),
    UnsignedNumber(u32),
    NumberArray(Vec<i32>),
    /// Comment on the last element
    ReallyCoolType(ItemDetailsFieldValue),
}

#[typeshare]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum AdvancedColors2 {
    /// This is a case comment
    String(String),
    Number(i32),
    NumberArray(Vec<i32>),
    /// Comment on the last element
    ReallyCoolType(ItemDetailsFieldValue),
}
