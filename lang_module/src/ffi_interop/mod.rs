/*!
Types defined in here are used to communicate between the language_module and the typeshare parser
The API may change at any time, so don't rely on it

 # Safety
 Any FFI type must be either converted into its safe counterpart or explicitly dropped
*/
#[cfg(feature = "ffi_v1")]
pub mod ffi_v1;
#[cfg(feature = "ffi_v1")]
#[no_mangle]
pub static TYPESHARE_FFI_VERSION: std::ffi::c_int = 1;
