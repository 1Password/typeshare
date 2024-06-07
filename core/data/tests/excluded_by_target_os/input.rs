#[typeshare]
pub enum TestEnum {
    Variant1,
    #[cfg(target_os = "ios")]
    Variant2,
    #[cfg(any(target_os = "ios", feature = "test"))]
    Variant3,
    #[cfg(all(target_os = "ios", feature = "test"))]
    Variant4,
}
