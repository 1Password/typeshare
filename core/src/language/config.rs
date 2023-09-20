use crate::language::TypeMapping;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::ops::Add;

pub trait LanguageConfig: Serialize + DeserializeOwned + Default + Add {
    fn default_file_name(&self) -> &str;

    fn type_mappings(&self) -> &TypeMapping;

    fn add_common_mappings(&mut self, type_mappings: TypeMapping);
    fn file_header(&self) -> Option<&str>;
}
