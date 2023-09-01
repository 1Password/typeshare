#[typeshare]
#[serde(tag = "type")]
pub enum SomeEnum {
    A,
    B { field1: String },
    C { field1: u32, field2: f32 },
    D { field3: Option<bool> },
}
