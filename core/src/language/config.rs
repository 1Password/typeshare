use std::ops::Add;

use serde::{de::DeserializeOwned, Serialize};

use crate::language::TypeMapping;

pub trait LanguageConfig: Serialize + DeserializeOwned + Default + Add {
    fn default_file_name(&self) -> &str;

    fn type_mappings(&self) -> &TypeMapping;

    fn add_common_mappings(&mut self, type_mappings: TypeMapping);
    fn file_header(&self) -> Option<&str>;
}
