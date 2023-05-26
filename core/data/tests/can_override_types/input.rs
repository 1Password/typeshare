#[typeshare]
#[serde(rename_all = "camelCase")]
struct OverrideStruct {
    // These annotations are intentionally inconsistent across languages
    #[typeshare(
        swift(type = "Int"),
        typescript(readonly, type = "any | undefined"),
        kotlin(type = "Int"), go(type = "uint"),
        scala(type = "Short")
    )]
    field_to_override: String,
}

#[typeshare]
#[serde(tag = "type", content = "content")]
enum OverrideEnum {
    UnitVariant,
    TupleVariant(String),
    #[serde(rename_all = "camelCase")]
    AnonymousStructVariant {
        #[typeshare(
            swift(type = "Int"),
            typescript(readonly, type = "any | undefined"),
            kotlin(type = "Int"), go(type = "uint"),
            scala(type = "Short")
        )]
        field_to_override: String
    }
}