#[typeshare]
#[serde(tag = "type", content = "content")]
pub enum GenericEnum<A, B> {
    VariantA(A),
    VariantB(B),
}

#[typeshare]
pub struct StructUsingGenericEnum {
    enum_field: GenericEnum<String, i16>,
}

#[typeshare]
#[serde(tag = "type", content = "content")]
pub enum GenericEnumUsingGenericEnum<T> {
    VariantC(GenericEnum<T, T>),
    VariantD(GenericEnum<&'static str, std::collections::HashMap<String, T>>),
    VariantE(GenericEnum<&'static str, u32>),
}

#[typeshare]
#[serde(tag = "type", content = "content")]
pub enum GenericEnumsUsingStructVariants<T, U> {
    VariantF { action: T },
    VariantG { action: T, response: U },
    VariantH { non_generic: i32 },
    VariantI { vec: Vec<T>, action: MyType<T, U> },
}
