#[typeshare]
#[derive(Serialize, Debug)]
#[serde(tag = "type", content = "content")]
pub enum SomeEnum {
    /// The associated String contains some opaque context
    Context(#[typeshare(serialized_as = "String")] SomeOtherType),
    Other(i32),
}
