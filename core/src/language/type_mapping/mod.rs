mod to_type;
mod type_mapping_value;

#[doc(inline)]
pub use crate::language::type_mapping::to_type::ToType;
#[doc(inline)]
pub use crate::language::type_mapping::type_mapping_value::TypeMappingValue;

use crate::parsed_types::Comment;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::mem;
use std::ops::{Add, Deref, DerefMut};

/// A mapping of Type to either another Rust Type or a Language Type
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TypeMapping(pub HashMap<String, TypeMappingValue>);
impl Add for TypeMapping {
    type Output = Self;

    fn add(mut self, overrides: Self) -> Self::Output {
        if self.0.is_empty() {
            return overrides;
        } else if overrides.0.is_empty() {
            return self;
        }
        for (key, value) in overrides.0 {
            self.0.insert(key, value);
        }
        self
    }
}
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
    pub fn get_comments(&self, key: &str) -> Option<&Comment> {
        self.0.get(key).map(|v| &v.doc)
    }

    pub fn make_all_rust_types(&mut self) {
        for (_, value) in self.0.iter_mut() {
            if value.to_type.is_lang_type() {
                let rust_type = mem::take(value.to_type.deref_mut());
                value.to_type = ToType::RustType(rust_type);
            }
        }
    }
}
