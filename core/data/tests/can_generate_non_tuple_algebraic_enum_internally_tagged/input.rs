#[typeshare]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum NonTupleAlgebraicEnum {
    VariantA { foo: u32 },
    VariantB { foo: u32, bar: String },
    VariantC {},
    VariantD,
}
