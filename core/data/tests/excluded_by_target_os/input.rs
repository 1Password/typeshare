#![cfg(feature = "online")]
#![allow(dead_code)]

use std::collection::HashMap;

#[typeshare]
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
