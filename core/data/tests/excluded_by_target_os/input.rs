#![cfg(feature = "online")]
#![allow(dead_code)]

use std::collection::HashMap;

#[typeshare]
#[serde(tag = "type", content = "content")]
pub enum TestEnum {
    Variant1,
    #[cfg(target_os = "ios")]
    Variant2,
    #[cfg(any(target_os = "ios", feature = "test"))]
    Variant3,
    #[cfg(all(target_os = "ios", feature = "test"))]
    Variant4,
    #[cfg(target_os = "android")]
    Variant5,
    #[cfg(target_os = "macos")]
    Variant7 {
        field1: String,
    },
    #[cfg(any(target_os = "android", target_os = "ios"))]
    Variant8,
    Variant9 {
        #[cfg(not(target_os = "macos"))]
        field1: String,
        field2: String,
    },
}

#[typeshare]
#[cfg(target_os = "ios")]
pub struct TestStruct;

#[typeshare]
#[cfg(target_os = "ios")]
type TypeAlias = String;

#[typeshare]
#[cfg(any(target_os = "ios", feature = "test"))]
pub enum Test {}

#[typeshare]
#[cfg(feature = "super")]
#[cfg(target_os = "android")]
pub enum SomeEnum {}

#[typeshare]
#[cfg(any(target_os = "ios", target_os = "android"))]
pub struct ManyStruct;

#[typeshare]
#[cfg(any(target_os = "android", target_os = "ios"))]
pub struct MultipleTargets;

#[typeshare]
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub struct DefinedTwice {
    field1: u64,
}

#[typeshare]
#[cfg(any(target_os = "android", target_os = "ios"))]
pub struct DefinedTwice {
    field1: String,
}

#[typeshare]
#[cfg(not(any(target_os = "wasm32", target_os = "ios")))]
pub struct Excluded;

#[typeshare]
#[cfg(not(target_os = "wasm32"))]
pub struct OtherExcluded;

#[typeshare]
#[cfg(not(target_os = "android"))]
pub struct AndroidExcluded;

#[typeshare]
#[cfg(all(feature = "my-feature", not(target_os = "ios")))]
pub struct NestedNotTarget1;

/// A struct with no target_os. Should be generated when
/// we use --target-os.
#[typeshare]
pub struct AlwaysAccept;

#[typeshare]
pub enum AlwaysAcceptEnum {
    Variant1,
    Variant2,
}
