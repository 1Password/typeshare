#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct ObjectNamedA {
    depends_on: String,
    age: i32,
    some_string_value: String,
}

#[typeshare]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DimensionFitValue {
    WrapContent,
    FitHeight,
}

#[typeshare]
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "kebab-case")]
pub enum DimensionValue {
    FixedSize(f32),
    Percentage(f32),
    Fit(DimensionFitValue),
}
