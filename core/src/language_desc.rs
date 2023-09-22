use crate::VERSION;
use semver::Version;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};


#[derive(Debug)]
#[repr(C)]
pub struct FFILanguageDescription {
    pub name: *const u8,
    pub name_len: usize,
    pub crate_version: *const u8,
    pub crate_version_len: usize,
    pub typeshare_version: *const u8,
    pub typeshare_version_len: usize,
    pub rust_version: *const u8,
    pub rust_version_len: usize,
    pub has_cli: bool,
}

impl Serialize for FFILanguageDescription {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut struct_serializer = serializer.serialize_struct("LanguageDescription", 4)?;
        struct_serializer.serialize_field("name", &self.name())?;
        struct_serializer.serialize_field("crate_version", &self.crate_version())?;
        struct_serializer.serialize_field("typeshare_version", &self.typeshare_version())?;
        struct_serializer.serialize_field("rust_version", &self.rust_version())?;
        struct_serializer.serialize_field("has_cli", &self.has_cli)?;
        struct_serializer.end()
    }
}
impl FFILanguageDescription {
    pub const fn new(
        name: &'static str,
        crate_version: &'static str,
        rust_version: &'static str,
        has_cli: bool,
    ) -> Self {
        Self {
            name: name.as_ptr(),
            name_len: name.len(),
            crate_version: crate_version.as_ptr(),
            crate_version_len: crate_version.len(),
            typeshare_version: VERSION.as_ptr(),
            typeshare_version_len: VERSION.len(),
            rust_version: rust_version.as_ptr(),
            rust_version_len: rust_version.len(),
            has_cli,
        }
    }
    pub fn name(&self) -> &'static str {
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.name, self.name_len))
        }
    }

    pub fn crate_version(&self) -> &'static str {
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                self.crate_version,
                self.crate_version_len,
            ))
        }
    }

    pub fn typeshare_version(&self) -> &'static str {
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                self.typeshare_version,
                self.typeshare_version_len,
            ))
        }
    }

    pub fn rust_version(&self) -> &'static str {
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                self.rust_version,
                self.rust_version_len,
            ))
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tabled", derive(tabled::Tabled))]
pub struct LanguageDescription {
    name: String,
    crate_version: String,
    typeshare_version: Version,
    #[cfg_attr(feature = "tabled", tabled(rename = "Rust Version Compiled With"))]
    rust_version: String,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    has_cli: bool,
}
impl LanguageDescription {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn crate_version(&self) -> &str {
        &self.crate_version
    }

    pub fn typeshare_version(&self) -> &Version {
        &self.typeshare_version
    }

    pub fn rust_version(&self) -> &str {
        &self.rust_version
    }
    pub fn has_cli(&self) -> bool {
        self.has_cli
    }
}
impl From<FFILanguageDescription> for LanguageDescription {
    fn from(ffi: FFILanguageDescription) -> Self {
        Self {
            name: ffi.name().to_string(),
            crate_version: ffi.crate_version().to_string(),
            typeshare_version: Version::parse(ffi.typeshare_version()).expect("Invalid version"),
            rust_version: ffi.rust_version().to_string(),
            has_cli: ffi.has_cli,
        }
    }
}
