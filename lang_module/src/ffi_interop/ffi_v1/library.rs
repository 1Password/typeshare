use crate::ffi_interop::ffi_v1::module_layout::LANGUAGE_MODULE_FUNCTION_NAME;
use crate::ffi_interop::ffi_v1::raw_parsed_data::RawParsedData;
use crate::ffi_interop::ffi_v1::{module_layout, FFILanguageLoggerConfig, FFIMap, FFIString};
use crate::language_module::LanguageModule;
use libloading::Library;
use log::{debug, error};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use thiserror::Error;

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
    #[error("Not Valid UTF-8: {0}")]
    IntoString(#[from] std::ffi::IntoStringError),
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
        let result = library.get::<*const i32>(module_layout::TYPESHARE_FFI_VERSION);
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

    pub fn init_logger(&self, config: impl Into<FFILanguageLoggerConfig>) -> Result<(), Error> {
        unsafe {
            let func = self
                .library
                .get::<module_layout::InitLoggerFunc>(module_layout::INIT_LOGGER_FUNCTION_NAME)?;
            func(config.into());
            Ok(())
        }
    }
    pub fn call_description(&self) -> Result<LanguageModule, Error> {
        unsafe {
            let func = self
                .library
                .get::<module_layout::LanguageModuleFunc>(LANGUAGE_MODULE_FUNCTION_NAME)?;
            Ok(func().into())
        }
    }
    pub fn call_build_types(
        &self,
        map: impl Into<FFIMap>,
        language_name: impl Into<FFIString>,
        raw_parsed_data: impl Into<RawParsedData>,
    ) -> Result<u32, Error> {
        unsafe {
            let func = self
                .library
                .get::<module_layout::BuildTypesFunc>(module_layout::BUILD_TYPES_FUNCTION_NAME)?;
            Ok(func(
                map.into(),
                language_name.into(),
                raw_parsed_data.into(),
            ))
        }
    }
    pub fn get_default_config(&self) -> Result<Option<String>, Error> {
        unsafe {
            let func = self.library.get::<module_layout::DefaultConfigFunc>(
                module_layout::DEFAULT_CONFIG_FUNCTION_NAME,
            );
            match func {
                Ok(ok) => {
                    let ok = ok();
                    if ok.is_empty() {
                        Ok(None)
                    } else {
                        Ok(Some(ok.into()))
                    }
                }
                Err(err) => {
                    // No default config function could be intentionally left out.
                    debug!("Unable to find default config function: {}", err);
                    Ok(None)
                }
            }
        }
    }
    ///
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn unload(self) -> Result<PathBuf, Error> {
        self.library.close()?;
        Ok(self.path.clone())
    }
}
