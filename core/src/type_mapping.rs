use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeMapping(HashMap<String, TypeMappingValue>);
impl Deref for TypeMapping {
    type Target = HashMap<String, TypeMappingValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl TypeMapping {
    pub fn get_comments(&self, key: &str) -> Option<&Vec<String>> {
        self.0.get(key).map(|v| &v.doc)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeMappingValue {
    pub to_type: String,
    pub doc: Vec<String>,
}

impl Into<String> for TypeMappingValue {
    fn into(self) -> String {
        self.to_type
    }
}
impl From<String> for TypeMappingValue {
    fn from(s: String) -> Self {
        TypeMappingValue {
            to_type: s,
            doc: Vec::new(),
        }
    }
}
impl FromStr for TypeMappingValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TypeMappingValue {
            to_type: s.to_string(),
            doc: Vec::new(),
        })
    }
}
impl Display for TypeMappingValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_type)
    }
}
const TO_TYPE: &str = "type";
const DOC: &str = "docs";

impl Serialize for TypeMappingValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if !self.doc.is_empty() {
            let mut s = serializer.serialize_struct("TypeMappingValue", 2)?;
            s.serialize_field(TO_TYPE, &self.to_type)?;
            if self.doc.len() == 1 {
                s.serialize_field(DOC, &self.doc[0])?;
            } else {
                s.serialize_field(DOC, &self.doc)?;
            }
            s.end()
        } else {
            serializer.serialize_str(&self.to_type)
        }
    }
}
struct TypeMappingValueVisitor;
impl<'de> Visitor<'de> for TypeMappingValueVisitor {
    type Value = TypeMappingValue;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a string or a struct with type and doc fields")
    }
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(TypeMappingValue {
            to_type: v.to_string(),
            doc: Vec::new(),
        })
    }
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(TypeMappingValue {
            to_type: v,
            doc: Vec::new(),
        })
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut to_type = None;
        let mut doc = None;
        while let Some(key) = map.next_key()? {
            match key {
                TO_TYPE => {
                    if to_type.is_some() {
                        return Err(serde::de::Error::duplicate_field(TO_TYPE));
                    }
                    to_type = Some(map.next_value()?);
                }
                DOC => {
                    if doc.is_some() {
                        return Err(serde::de::Error::duplicate_field(DOC));
                    }
                    if let Ok(v) = map.next_value::<String>() {
                        doc = Some(vec![v]);
                    } else {
                        doc = Some(map.next_value()?);
                    }
                }
                _ => {
                    return Err(serde::de::Error::unknown_field(key, &[TO_TYPE, DOC]));
                }
            }
        }
        let to_type = to_type.ok_or_else(|| serde::de::Error::missing_field(TO_TYPE))?;
        Ok(TypeMappingValue {
            to_type,
            doc: doc.unwrap_or_default(),
        })
    }
}
impl<'de> Deserialize<'de> for TypeMappingValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(TypeMappingValueVisitor)
    }
}
