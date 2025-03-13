/// This struct has a unit field
#[typeshare]
#[serde(default, rename_all = "camelCase")]
struct StructHasVoidType {
    this_is_a_unit: (),
}

/// This enum has a variant associated with unit data
#[typeshare]
#[serde(default, rename_all = "camelCase", tag = "type", content = "content")]
enum EnumHasVoidType {
    HasAUnit(()),
}
