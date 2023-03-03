#[typeshare]
pub struct SomeStruct {
    #[typeshare(typescript(readonly))]
    field_a: u32,
}
