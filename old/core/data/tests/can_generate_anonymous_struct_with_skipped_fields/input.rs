/// Enum keeping track of who autofilled a field
#[typeshare]
#[serde(tag = "type", content = "content")]
pub enum AutofilledBy {
    /// This field was autofilled by us
    Us {
        /// The UUID for the fill
        uuid: String,
    },
    /// Something else autofilled this field
    SomethingElse {
        /// The UUID for the fill
        uuid: String,
        /// Some other thing
        #[typeshare(skip)]
        thing: i32,
    },
}
