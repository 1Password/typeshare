use libloading::{Library};
use log::debug;
use std::ffi::{OsStr};
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use thiserror::Error;
use typeshare_core::FFILanguageDescription;
type DescriptionFunc = unsafe extern "C" fn() -> FFILanguageDescription;
type DefaultConfigFunc = unsafe extern "C" fn() -> *const c_char;
static DESCRIPTION_FUNCTION_NAME: &[u8] = b"description";
static DEFAULT_CONFIG_FUNCTION_NAME: &[u8] = b"generate_default_config";
static TYPESHARE_FFI_VERSION: &[u8] = b"TYPESHARE_FFI_VERSION\0";
#[derive(Debug, Error)]
pub enum Error {
    #[error("An error occurred while loading the library: {0}")]
    IO(std::io::Error),
    #[error("An error occurred while loading the library: {0}")]
    LibLoading(#[from] libloading::Error),
    #[error("Unsupported library version {0:?}: Closed Result {1:?}")]
    UnsupportedVersion(i32, Result<(), libloading::Error>),
    #[error("Unable to pull TYPESHARE_FFI_VERSION. {0:?}: Closed Result {1:?}")]
    MissingVersion(libloading::Error, Result<(), libloading::Error>),
    #[error("Not Valid UTF-8: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}

#[derive(Debug)]
pub struct LanguageLibrary {
    library: Library,
    path: PathBuf,
}

impl LanguageLibrary {
    pub fn can_load(path: &impl AsRef<Path>) -> bool {
        if path.as_ref().is_dir() || !path.as_ref().exists() {
            return false;
        }
        path.as_ref()
            .extension()
            .map(|v| v == OsStr::new("so") || v == OsStr::new("dll") || v == OsStr::new("dylib"))
            .unwrap_or(false)
    }
    /// Loads a library from the given path.
    /// # Safety
    /// Remember to call `unload` when you are done with the library.
    pub unsafe fn load(path: PathBuf) -> Result<Self, Error> {
        debug!("Loading library from {:?}", path);

        let library = Library::new(&path)?;
        let result = library.get::<*const i32>(TYPESHARE_FFI_VERSION);
        match result {
            Ok(ok) => {
                let ok = **ok;
                if ok != 1 {
                    let close_result = library.close();
                    return Err(Error::UnsupportedVersion(ok, close_result));
                }
            }
            Err(err) => {
                let close_result = library.close();
                return Err(Error::MissingVersion(err, close_result));
            }
        }
        debug!("Loaded library {:?}", library);
        Ok(Self { library, path })
    }
    pub fn call_description(&self) -> Result<FFILanguageDescription, Error> {
        unsafe {
            let func = self
                .library
                .get::<DescriptionFunc>(DESCRIPTION_FUNCTION_NAME)?;
            Ok(func())
        }
    }
    pub fn get_default_config(&self) -> Result<Option<String>, Error> {
        unsafe {
            let func = self
                .library
                .get::<DefaultConfigFunc>(DEFAULT_CONFIG_FUNCTION_NAME)?;
            let c_string = func();
            if c_string.is_null() {
                return Ok(None);
            }
            let c_string = std::ffi::CStr::from_ptr(c_string);
            let string = c_string.to_str()?;
            Ok(Some(string.to_string()))
        }
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn unload(self) -> Result<PathBuf, Error> {
        self.library.close()?;
        Ok(self.path)
    }
}
