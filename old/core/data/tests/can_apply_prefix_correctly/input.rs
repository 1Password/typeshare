#[typeshare]
pub struct ItemDetailsFieldValue {
    hello: String,
}

#[typeshare]
#[serde(tag = "t", content = "c")]
pub enum AdvancedColors {
    String(String),
    Number(i32),
    NumberArray(Vec<i32>),
    ReallyCoolType(ItemDetailsFieldValue),
    ArrayReallyCoolType(Vec<ItemDetailsFieldValue>),
    DictionaryReallyCoolType(HashMap<String, ItemDetailsFieldValue>),
}
