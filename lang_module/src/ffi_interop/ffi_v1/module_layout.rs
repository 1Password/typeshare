/*!
# Functions And Function Names Expected to be Exported by a Language Module
 */
#![allow(unused)]
use crate::ffi_interop::ffi_v1::raw_parsed_data::RawParsedData;
use crate::ffi_interop::ffi_v1::{
    FFIArgumentRef, FFIArray, FFILanguageLoggerConfig, FFILanguageModule, FFIMap, FFIString,
};
use crate::language_logger::LanguageLoggerConfig;

pub static LANGUAGE_MODULE_FUNCTION_NAME: &[u8] = b"language_module";
pub type LanguageModuleFunc = unsafe extern "C" fn() -> FFILanguageModule;

pub static INIT_LOGGER_FUNCTION_NAME: &[u8] = b"init_logger";
pub type InitLoggerFunc = unsafe extern "C" fn(FFILanguageLoggerConfig);

pub static LANGUAGE_CONFIG_FUNCTION_NAME: &[u8] = b"language_config";
pub type LanguageConfig = unsafe extern "C" fn() -> FFIArray<FFIArgumentRef>;
pub static DEFAULT_CONFIG_FUNCTION_NAME: &[u8] = b"default_config";
pub type DefaultConfigFunc = unsafe extern "C" fn() -> FFIString;

pub static BUILD_TYPES_FUNCTION_NAME: &[u8] = b"build_types";
pub type BuildTypesFunc = unsafe extern "C" fn(FFIMap, FFIString, RawParsedData) -> u32;

pub static TYPESHARE_FFI_VERSION: &[u8] = b"TYPESHARE_FFI_VERSION\0";
