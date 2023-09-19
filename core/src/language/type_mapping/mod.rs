mod to_type;
mod type_mapping_value;

#[doc(inline)]
pub use crate::language::type_mapping::to_type::ToType;
#[doc(inline)]
pub use crate::language::type_mapping::type_mapping_value::TypeMappingValue;

use crate::language::Comment;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;

/// A mapping of Type to either another Rust Type or a Language Type
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TypeMapping(pub HashMap<String, TypeMappingValue>);
impl Serialize for TypeMapping {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for TypeMapping {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(TypeMapping(HashMap::deserialize(deserializer)?))
    }
}
impl Deref for TypeMapping {
    type Target = HashMap<String, TypeMappingValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl TypeMapping {
    /// Get the comments for a TypeMapping key
    pub fn get_comments(&self, key: &str) -> Option<&Comment<'static>> {
        self.0.get(key).map(|v| &v.doc)
    }
}
