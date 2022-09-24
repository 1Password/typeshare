#[typeshare]
pub struct OtherType {}

/// This is a comment.
#[typeshare]
pub struct Person {
    pub name: String,
    pub age: u8,
    #[serde(rename = "extraSpecialFieldOne")]
    pub extra_special_field1: i32,
    #[serde(rename = "extraSpecialFieldTwo")]
    pub extra_special_field2: Option<Vec<String>>,
    #[serde(rename = "nonStandardDataType")]
    pub non_standard_data_type: OtherType,
    #[serde(rename = "nonStandardDataTypeInArray")]
    pub non_standard_data_type_in_array: Option<Vec<OtherType>>,
}
