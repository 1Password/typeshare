#[typeshare]
pub struct ItemDetailsFieldValue {}

#[typeshare]
#[serde(rename_all = "camelCase", tag = "type", content = "content")]
pub enum AdvancedColors {
    String(String),
    Number(i32),
    #[serde(rename = "number-array")]
    NumberArray(Vec<i32>),
    #[serde(rename = "reallyCoolType")]
    ReallyCoolType(ItemDetailsFieldValue),
}
