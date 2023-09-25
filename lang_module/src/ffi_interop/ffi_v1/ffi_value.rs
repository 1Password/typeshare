#![allow(clippy::wrong_self_convention)]

use crate::ffi_interop::ffi_v1::ffi_map::FFIMap;
use crate::ffi_interop::ffi_v1::ffi_string::FFIString;
use crate::ffi_interop::ffi_v1::{FFIArray, FFIType};
use serde::{Serialize, Serializer};
use std::fmt::Debug;
use typeshare_core::value::{Number, Value};

#[derive(Debug, Clone)]
#[repr(C)]
pub enum FFIValue {
    String(FFIString),
    Array(FFIArray<FFIValue>),
    Map(FFIMap),
    Bool(bool),
    Int(u64),
    SignedInt(i64),
    Float(f64),
}
impl From<Value> for FFIValue {
    fn from(value: Value) -> Self {
        match value {
            Value::String(string) => Self::String(string.into()),
            Value::Array(array) => Self::Array(array.try_into().unwrap()),
            Value::Object(map) => Self::Map(map.try_into().unwrap()),
            Value::Bool(bool) => Self::Bool(bool),
            Value::Null => Value::String("null".to_string()).into(),
            Value::Number(n) => match n {
                Number::PosInt(p) => FFIValue::Int(p),
                Number::SignedInt(v) => FFIValue::SignedInt(v),
                Number::Float(f) => FFIValue::Float(f),
            },
        }
    }
}
impl FFIType for FFIValue {
    type SafeType = Value;
}
impl Serialize for FFIValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            FFIValue::String(string) => string.serialize(serializer),
            FFIValue::Array(array) => array.serialize(serializer),
            FFIValue::Map(map) => map.serialize(serializer),
            FFIValue::Bool(bool) => serializer.serialize_bool(*bool),
            FFIValue::Int(int) => serializer.serialize_u64(*int),
            FFIValue::SignedInt(int) => serializer.serialize_i64(*int),
            FFIValue::Float(float) => serializer.serialize_f64(*float),
        }
    }
}
impl PartialEq for FFIValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (FFIValue::String(string), FFIValue::String(other_string)) => string == other_string,
            (FFIValue::Array(array), FFIValue::Array(other_array)) => array == other_array,
            (FFIValue::Map(map), FFIValue::Map(other_map)) => map == other_map,
            (FFIValue::Bool(bool), FFIValue::Bool(other_bool)) => bool == other_bool,
            (FFIValue::Int(int), FFIValue::Int(other_int)) => int == other_int,
            (FFIValue::SignedInt(int), FFIValue::SignedInt(other_int)) => int == other_int,
            (FFIValue::Float(float), FFIValue::Float(other_float)) => float == other_float,
            _ => false,
        }
    }
}
impl From<FFIValue> for Value {
    fn from(value: FFIValue) -> Self {
        match value {
            FFIValue::String(string) => Value::String(string.into()),
            FFIValue::Array(array) => Value::Array(array.try_into().unwrap()),
            FFIValue::Map(map) => Value::Object(map.try_into().unwrap()),
            FFIValue::Bool(bool) => Value::Bool(bool),

            FFIValue::Int(i) => i.into(),
            FFIValue::SignedInt(v) => v.into(),
            FFIValue::Float(v) => v.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OptionalFFIValue {
    Some(FFIValue),
    None,
}
impl FFIType for OptionalFFIValue {
    type SafeType = Option<Value>;
}
impl From<OptionalFFIValue> for Option<Value> {
    fn from(value: OptionalFFIValue) -> Self {
        match value {
            OptionalFFIValue::Some(string) => Some(string.try_into().unwrap()),
            OptionalFFIValue::None => None,
        }
    }
}

impl From<Option<Value>> for OptionalFFIValue {
    fn from(value: Option<Value>) -> Self {
        match value {
            Some(value) => Self::Some(value.try_into().unwrap()),
            None => Self::None,
        }
    }
}
impl Serialize for OptionalFFIValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            OptionalFFIValue::Some(string) => string.serialize(serializer),
            OptionalFFIValue::None => serializer.serialize_none(),
        }
    }
}
