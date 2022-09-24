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
        thing: i32,
    },
}

/// This is a comment (yareek sameek wuz here)
#[typeshare]
#[serde(tag = "type", content = "content")]
pub enum EnumWithManyVariants {
    UnitVariant,
    TupleVariantString(String),
    AnonVariant { uuid: String },
    TupleVariantInt(i32),
    AnotherUnitVariant,
    AnotherAnonVariant { uuid: String, thing: i32 },
}
