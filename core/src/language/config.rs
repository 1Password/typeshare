use crate::language::TypeMapping;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub trait LanguageConfig: Serialize + DeserializeOwned + Default {
    fn default_file_name(&self) -> &str;

    fn type_mappings(&self) -> &TypeMapping;

    fn file_header(&self) -> Option<&str>;
}
#[derive(Serialize, Deserialize, Default)]
pub struct CommonConfig {
    /// Any Value inside the Type Mapping will be assumed to be a Rust Type
    pub type_mappings: TypeMapping,
}
