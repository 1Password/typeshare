#![cfg(feature = "online")]
#![allow(dead_code)]
#![cfg(any(target_os = "android", feature = "testing"))]
#![cfg(target_os = "wasm32")]

#[typeshare]
pub struct IgnoredUnlessAndroid;
