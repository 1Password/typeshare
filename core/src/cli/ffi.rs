use std::ffi::{c_char, CStr, CString};
use std::mem;
use std::str::Utf8Error;

#[repr(C)]
pub enum Error {
    Utf8Error,
}
pub fn to_c_string_array(args: &[&str]) -> (*const *const c_char, usize) {
    let mut result = Vec::with_capacity(args.len());

    for arg in args {
        let c_str = CString::new(arg.to_string()).unwrap();
        result.push(c_str.as_ptr());
        mem::forget(c_str)
    }
    let result = result.into_boxed_slice();
    let result = result.as_ptr();
    mem::forget(result);
    (result, args.len())
}
pub fn to_args_array(args: *const *const c_char, size: usize) -> Result<Vec<String>, Utf8Error> {
    let mut result = Vec::new();
    let args = unsafe { std::slice::from_raw_parts(args, size) };
    for arg in args {
        let arg = unsafe { CStr::from_ptr(*arg) };
        result.push(arg.to_str()?.to_string());
    }
    Ok(result)
}
mod ffi_tests {
    #[test]
    pub fn test() {
        let mut vec_args = vec!["test", "test2"];
        let (args, size) = super::to_c_string_array(&vec_args);
        vec_args[0] = "test3";
        let args = unsafe { super::to_args_array(args, size) }.unwrap();
        assert_eq!(args, vec!["test", "test2"]);
    }
}
