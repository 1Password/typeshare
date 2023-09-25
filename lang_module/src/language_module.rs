use semver::Version;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIs};

#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize, Display, EnumIs)]
#[repr(C)]
pub enum FeatureFlags {
    BuildEnums,
    BuildStructs,
    BuildTypeAliases,
    RequiresMultipleFiles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageModule {
    pub language_name: String,
    pub language_module_version: String,
    pub typeshare_version: Version,
    pub rust_version: String,
    pub ffi_version: i32,
    pub authors: Vec<String>,
    pub website: Option<String>,
    pub feature_flags: Vec<FeatureFlags>,
}

mod ffi_v1 {
    use crate::ffi_interop::ffi_v1::FFILanguageModule;
    use crate::language_module::LanguageModule;
    use semver::Version;

    impl From<FFILanguageModule> for LanguageModule {
        fn from(value: FFILanguageModule) -> Self {
            let FFILanguageModule {
                language_name,
                language_module_version,
                typeshare_version,
                rust_version,
                ffi_version,
                authors,
                website,
                feature_flags,
            } = value;
            let language_name = language_name.to_string();
            let language_module_version = language_module_version.to_string();
            let typeshare_version = Version::parse(typeshare_version.as_ref()).unwrap();
            let rust_version = rust_version.to_string();
            let feature_flags = feature_flags.try_into().unwrap();
            let authors = authors.try_into().unwrap();
            let website = website.into();
            Self {
                language_name,
                language_module_version,
                typeshare_version,
                rust_version,
                ffi_version,
                authors,
                website,
                feature_flags,
            }
        }
    }
}
