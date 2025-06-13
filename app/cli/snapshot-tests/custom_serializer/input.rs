#[typeshare(kotlin = "Serializer(CustomSerializer)")]
pub struct BestHockeyTeams {
    PittsburghPenguins: u32,
    Lies: String,
}

#[typeshare(kotlin = "Serializer(CustomSerializer)")]
#[serde(tag = "type", content = "content")]
pub enum Phrase {
    ScanSetupCode,
    TotpSecondsRemaining(String),
    NestedAnonymousStruct {
        nested_field: String,
        another_nested_field: String
    }
}
