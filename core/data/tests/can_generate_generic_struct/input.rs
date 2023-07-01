#[typeshare]
pub struct GenericStruct<A, B> {
    field_a: A,
    field_b: Vec<B>,
}

#[typeshare]
pub struct UnusedGenericsStruct<A, B> {
    field_a: f32,
    field_b: f32,
    _phantom_data: std::marker::PhantomData<A, B>,
}

#[typeshare]
pub struct UnusedGenericsEmptyStruct<A, B, C> {}

#[typeshare]
#[serde(tag = "type", content = "content")]
pub enum EnumUsingGenericStruct {
    VariantA(GenericStruct<String, f32>),
    VariantB(GenericStruct<&'static str, i32>),
    VariantC(GenericStruct<&'static str, bool>),
    VariantD(GenericStructUsingGenericStruct<()>),
}

#[typeshare]
pub struct GenericStructUsingGenericStruct<T> {
    struct_field: GenericStruct<String, T>,
    second_struct_field: GenericStruct<T, String>,
    third_struct_field: GenericStruct<T, Vec<T>>,
}
