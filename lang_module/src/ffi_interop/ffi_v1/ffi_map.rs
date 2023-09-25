#![allow(clippy::wrong_self_convention)]

use crate::ffi_interop::ffi_v1::ffi_array::FFIArray;
use crate::ffi_interop::ffi_v1::ffi_string::FFIString;
use crate::ffi_interop::ffi_v1::ffi_value::FFIValue;
use crate::ffi_interop::ffi_v1::FFIType;
use serde::ser::SerializeMap;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Formatter};
use typeshare_core::value::Value;

#[derive(PartialEq, Debug)]
#[repr(C)]
pub struct FFIEntry {
    pub key: FFIString,
    pub value: FFIValue,
}
impl FFIType for FFIEntry {
    type SafeType = (String, Value);
}
impl From<FFIEntry> for (String, Value) {
    fn from(value: FFIEntry) -> Self {
        let FFIEntry { key, value } = value;
        let key = key.into();
        let value = value.try_into().unwrap();
        (key, value)
    }
}
impl From<(String, Value)> for FFIEntry {
    fn from((key, value): (String, Value)) -> Self {
        let key = FFIString::from(key);

        let value = FFIValue::try_from(value).unwrap();
        Self { key, value }
    }
}
impl Clone for FFIEntry {
    fn clone(&self) -> Self {
        let FFIEntry { key, value } = self;
        let key = key.clone();
        let value = value.clone();
        Self { key, value }
    }
}
#[derive(Clone)]
#[repr(transparent)]
pub struct FFIMap(FFIArray<FFIEntry>);
impl Serialize for FFIMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let FFIMap(array) = self;
        let array = array.as_box();
        let mut map = serializer.serialize_map(Some(array.len()))?;
        for entry in array.iter() {
            map.serialize_entry(&entry.key, &entry.value)?;
        }
        map.end()
    }
}
impl TryFrom<HashMap<String, Value>> for FFIMap {
    type Error = <FFIEntry as TryFrom<(String, Value)>>::Error;

    fn try_from(value: HashMap<String, Value>) -> Result<Self, Self::Error> {
        let mut entries = Vec::with_capacity(value.len());
        for (key, value) in value.into_iter() {
            entries.push((key, value));
        }
        FFIArray::try_from(entries).map(Self)
    }
}
impl TryFrom<FFIMap> for HashMap<String, Value> {
    type Error = <FFIEntry as TryFrom<(String, Value)>>::Error;

    fn try_from(value: FFIMap) -> Result<Self, Self::Error> {
        let FFIMap(array) = value;
        let entries: Vec<(String, Value)> = array.try_into()?;
        Ok(HashMap::from_iter(entries))
    }
}
impl TryFrom<BTreeMap<String, Value>> for FFIMap {
    type Error = <FFIEntry as TryFrom<(String, Value)>>::Error;

    fn try_from(value: BTreeMap<String, Value>) -> Result<Self, Self::Error> {
        let mut entries = Vec::with_capacity(value.len());
        for (key, value) in value.into_iter() {
            entries.push((key, value));
        }
        FFIArray::try_from(entries).map(Self)
    }
}
impl TryFrom<FFIMap> for BTreeMap<String, Value> {
    type Error = <FFIEntry as TryFrom<(String, Value)>>::Error;

    fn try_from(value: FFIMap) -> Result<Self, Self::Error> {
        let FFIMap(array) = value;
        let entries: Vec<(String, Value)> = array.try_into()?;
        Ok(BTreeMap::from_iter(entries))
    }
}
impl PartialEq for FFIMap {
    fn eq(&self, other: &Self) -> bool {
        let FFIMap(array) = self;
        let FFIMap(other_array) = other;
        let array_one = array.as_box();
        let array_two = other_array.as_box();

        for array_one_value in array_one.iter() {
            let option = array_two
                .iter()
                .find(|v| v.key.as_ref() == array_one_value.key.as_ref());

            if let Some(other_entry) = option {
                if array_one_value.value != other_entry.value {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}
impl Debug for FFIMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let FFIMap(array) = self;
        let mut map = f.debug_map();
        let array = array.as_box();
        for entry in array.iter() {
            map.entry(&entry.key, &entry.value);
        }
        map.finish()
    }
}
#[cfg(test)]
mod map_tests {
    use crate::ffi_interop::ffi_v1::ffi_map::FFIMap;
    use std::collections::HashMap;
    use typeshare_core::value::Value;

    #[test]
    pub fn normal_map() {
        for i in 0..10 {
            let mut map = HashMap::new();
            map.insert("hello".to_string(), Value::String("world".to_string()));
            map.insert("foo".to_string(), Value::Bool(true));
            map.insert("bar".to_string(), Value::Number(i.into()));
            map.insert("baz".to_string(), Value::Number((i as i64).into()));
            map.insert("qux".to_string(), Value::Number((i as f64).into()));
            map.insert(
                "quuz".to_string(),
                Value::Array(vec![Value::String("hello".to_string()), Value::Bool(true)]),
            );
            map.insert(
                "corge".to_string(),
                Value::Object(
                    vec![
                        ("hello".to_string(), Value::String("world".to_string())),
                        ("foo".to_string(), Value::Bool(true)),
                    ]
                    .into_iter()
                    .collect(),
                ),
            );

            let ffi_map: FFIMap = map.clone().try_into().unwrap();
            println!("FFIMap: {:#?}", ffi_map);
            let map: HashMap<String, Value> = ffi_map.try_into().unwrap();
            assert_eq!(map, map);
        }
    }
}
