/*!
FFI Version 1 Layout


*/
use log::LevelFilter;
use serde::Serialize;
use std::collections::BTreeMap;
use std::ffi::c_int;

pub mod argument;

pub mod ffi_array;
pub mod ffi_map;
pub mod ffi_string;

pub mod ffi_value;
#[cfg(feature = "libloading")]
pub mod library;
pub mod module_layout;
pub mod raw_parsed_data;

use crate::language_logger::LanguageLoggerConfig;
use crate::language_module::FeatureFlags;
pub use argument::{FFIArgumentRef, FFIArgumentType, FFICLIArgument};
pub use ffi_array::FFIArray;
pub use ffi_map::FFIMap;
pub use ffi_string::{FFIString, OptionalFFIString};
pub use ffi_value::FFIValue;
use typeshare_core::value::{ToFromValue, Value};

#[derive(Debug, Clone)]
#[repr(C)]
pub enum Error {
    IO(FFIString),
    Internal(FFIString),
}

macro_rules! ffi_type_for_primitive_copy {
    ($($t:ty),*) =>{
        $(
            impl FFIType for $t {
                type SafeType = $t;
            }
        )*

    };
}
#[derive(Debug, Serialize)]
#[repr(C)]
pub struct FFILanguageModule {
    pub language_name: FFIString,
    pub language_module_version: FFIString,
    pub typeshare_version: FFIString,
    pub rust_version: FFIString,
    pub ffi_version: c_int,
    pub authors: FFIArray<FFIString>,
    pub website: OptionalFFIString,
    pub feature_flags: FFIArray<FeatureFlags>,
}

impl FFILanguageModule {
    pub fn new(
        language_name: String,
        language_module_version: String,
        feature_flags: Vec<FeatureFlags>,
        authors: Vec<String>,
        website: Option<String>,
    ) -> Self {
        let language_name = FFIString::from(language_name);
        let language_module_version = FFIString::from(language_module_version);
        let typeshare_version = FFIString::try_from(env!("CARGO_PKG_VERSION").to_string()).unwrap();
        let authors = FFIArray::try_from(authors).unwrap();
        let website = OptionalFFIString::try_from(website).unwrap();
        let rust_version = FFIString::try_from(env!("VERGEN_RUSTC_SEMVER").to_string()).unwrap();
        Self {
            language_name,
            language_module_version,
            typeshare_version,
            rust_version,
            ffi_version: 1,
            authors,
            website,
            feature_flags: FFIArray::try_from(feature_flags).unwrap(),
        }
    }
}

/// A Type that implements this Should be convertible to and from its safe counterpart
///
/// Then it should handle dropping itself if necessary
pub trait FFIType: Sized {
    type SafeType: TryFrom<Self> + TryInto<Self>;
}

ffi_type_for_primitive_copy!(i8, i16, i32, i64, i128, isize);
ffi_type_for_primitive_copy!(u8, u16, u32, u64, u128, usize);
ffi_type_for_primitive_copy!(bool, f32, f64, FeatureFlags);
/// Creates a [toml::Value] from a [FFIMap]
pub fn parse_ffi_map<D: ToFromValue>(config: FFIMap) -> Result<D, FFIString> {
    let result = TryInto::<BTreeMap<String, Value>>::try_into(config)
        .map(Value::Object)
        .unwrap();
    ToFromValue::from_value(result).map_err(|e| FFIString::from(e.to_string()))
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct FFILanguageLoggerConfig {
    pub language_crate_level: LevelFilter,
    pub typeshare_core_level: LevelFilter,
}

impl From<LanguageLoggerConfig> for FFILanguageLoggerConfig {
    fn from(value: LanguageLoggerConfig) -> Self {
        let LanguageLoggerConfig {
            language_crate_level,
            typeshare_core_level,
        } = value;
        Self {
            language_crate_level,
            typeshare_core_level,
        }
    }
}
