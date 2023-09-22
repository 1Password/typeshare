mod to_type;
mod type_mapping_value;

use std::{
    collections::HashMap,
    mem,
    ops::{Add, Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

#[doc(inline)]
pub use crate::language::type_mapping::to_type::ToType;
#[doc(inline)]
pub use crate::language::type_mapping::type_mapping_value::TypeMappingValue;
use crate::parsed_types::Comment;

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
impl DerefMut for TypeMapping {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
#[cfg(test)]
mod tests {
    use crate::language::type_mapping::ToType;
    use crate::language::{TypeMapping, TypeMappingValue};
    use crate::parsed_types::Comment;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Test {
        type_mapping: TypeMapping,
    }
    #[test]
    pub fn basic_deserialize() {
        let mut test = Test::default();
        test.type_mapping
            .insert("usize".to_string(), "usize".to_string().into());
        test.type_mapping
            .insert("String".to_string(), "String".to_string().into());
        test.type_mapping
            .insert("Vec".to_string(), "Vec".to_string().into());

        let serialized = toml::to_string_pretty(&test).unwrap();
        println!("{}", serialized);
        let deserialized: Test = toml::from_str(&serialized).unwrap();
        assert_eq!(test, deserialized);
    }
    #[test]
    pub fn deserialize_with_comment() {
        let mut test = Test::default();
        test.type_mapping
            .insert("usize".to_string(), "usize".to_string().into());
        test.type_mapping
            .insert("String".to_string(), "String".to_string().into());
        test.type_mapping.insert(
            "Vec".to_string(),
            TypeMappingValue {
                to_type: "Vec".to_string().into(),
                doc: Comment::new_single(
                    "This is a comment",
                    crate::parsed_types::CommentLocation::Field,
                ),
            },
        );

        let serialized = toml::to_string_pretty(&test).unwrap();
        println!("{}", serialized);
        let deserialized = toml::from_str::<Test>(&serialized);
        println!("{:#?}", deserialized);
        assert!(deserialized.is_ok());
    }
    #[test]
    pub fn deserialize_with_complex_to_type() {
        let mut test = Test::default();
        test.type_mapping
            .insert("usize".to_string(), "usize".to_string().into());
        test.type_mapping
            .insert("String".to_string(), "String".to_string().into());
        test.type_mapping.insert(
            "Vec".to_string(),
            TypeMappingValue {
                to_type: ToType::RustType("Vec".to_string()),
                doc: Comment::new_single(
                    "This is a comment",
                    crate::parsed_types::CommentLocation::Field,
                ),
            },
        );

        let serialized = toml::to_string_pretty(&test).unwrap();
        println!("{}", serialized);
        let deserialized = toml::from_str::<Test>(&serialized);
        println!("{:#?}", deserialized);
        assert!(deserialized.is_ok());
    }
}
