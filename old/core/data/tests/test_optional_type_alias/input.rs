#[typeshare]
pub type OptionalU32 = Option<u32>;

#[typeshare]
pub struct OptionalU16(Option<u16>);

#[typeshare]
pub struct FooBar {
    foo: OptionalU32,
    bar: OptionalU16,
}
