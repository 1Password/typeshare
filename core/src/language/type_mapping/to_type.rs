use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToType {
    /// Maps a Rust Type to the type within the language.
    /// Should only be used in the Language Specific TypeMapping
    /// ```toml
    /// [type_mapping]
    /// "i64" = "BigInt"
    /// ```
    LangType(String),
    /// Maps a Rust Type to another Rust type that needs to be pulled from the TypeMapping
    /// # Warning
    /// This is not currently implemented
    /// // TODO: Implement this
    ///
    /// # Example
    /// ```toml
    /// [type_mapping]
    /// "MyType" = { type = "Rust", content = "MyOtherType" }
    /// ```
    RustType(String),
}
impl Serialize for ToType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ToType::LangType(s) => serializer.serialize_str(s),
            ToType::RustType(rust_type) => {
                let mut s = serializer.serialize_struct("ToType", 2)?;
                s.serialize_field("type", "Rust")?;
                s.serialize_field("content", rust_type)?;
                s.end()
            }
        }
    }
}
struct ToTypeVisitor;
impl<'de> Visitor<'de> for ToTypeVisitor {
    type Value = ToType;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a string or a struct with type and content fields")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ToType::LangType(v.to_string()))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut type_name = None;
        let mut content = None;
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "type" => {
                    type_name = Some(map.next_value::<String>()?);
                }
                "content" => {
                    content = Some(map.next_value()?);
                }
                _ => {
                    return Err(serde::de::Error::unknown_field(&key, &["type", "content"]));
                }
            }
        }
        let type_name = type_name.ok_or_else(|| serde::de::Error::missing_field("type"))?;
        let content = content.ok_or_else(|| serde::de::Error::missing_field("content"))?;
        if type_name == "Rust" {
            Ok(ToType::RustType(content))
        } else {
            Err(serde::de::Error::custom(format!(
                "unknown type: {}",
                type_name
            )))
        }
    }
}
impl<'de> Deserialize<'de> for ToType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ToTypeVisitor)
    }
}
impl Deref for ToType {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        match self {
            ToType::LangType(s) => s,
            ToType::RustType(s) => s,
        }
    }
}
impl Display for ToType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ToType::LangType(s) => write!(f, "{}", s),
            ToType::RustType(s) => write!(f, "{}", s),
        }
    }
}

impl Into<String> for ToType {
    fn into(self) -> String {
        match self {
            ToType::LangType(s) => s,
            ToType::RustType(s) => s,
        }
    }
}
