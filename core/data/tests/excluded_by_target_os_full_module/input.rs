#![cfg(feature = "online")]
#![allow(dead_code)]
#![cfg(any(target_os = "android", feature = "testing"))]
#![cfg(target_os = "android")]

#[typeshare]
pub struct IgoredUnlessAndroid;
