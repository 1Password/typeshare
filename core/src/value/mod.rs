mod serde_wraps;
mod serialize_and_deserialize;

use std::collections::BTreeMap;
use std::fmt::Display;
use strum::{Display, EnumIs, IntoStaticStr};
use thiserror::Error;
macro_rules! from_number {
    ($ty:ty, $variant:ident as $as_t:ty) => {
        impl From<$ty> for Number {
            fn from(value: $ty) -> Self {
                Self::$variant(value as $as_t)
            }
        }
        impl From<$ty> for Value {
            fn from(value: $ty) -> Self {
                Self::Number(value.into())
            }
        }
    };
}
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Number {
    PosInt(u64),
    SignedInt(i64),
    Float(f64),
}
from_number!(u8, PosInt as u64);
from_number!(i8, SignedInt as i64);
from_number!(u16, PosInt as u64);
from_number!(i16, SignedInt as i64);
from_number!(u32, PosInt as u64);
from_number!(i32, SignedInt as i64);
from_number!(u64, PosInt as u64);
from_number!(i64, SignedInt as i64);
from_number!(f64, Float as f64);
from_number!(f32, Float as f64);
#[derive(Debug, Error)]
pub enum ToValueError {
    #[error("Custom deserialize error: {0}")]
    CustomSerializeError(String),
    #[error("Key must be a string")]
    KeyMustBeString,
    #[error("Key not present")]
    KeyNotPresent,
}
impl serde::ser::Error for ToValueError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::CustomSerializeError(msg.to_string())
    }
}
#[derive(Debug, Error)]
pub enum FromValueError {
    #[error("Custom deserialize error: {0}")]
    CustomDeserializeError(String),
}

impl serde::de::Error for FromValueError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::CustomDeserializeError(msg.to_string())
    }
}

pub trait ToFromValue {
    fn from_value(value: Value) -> Result<Self, FromValueError>
    where
        Self: Sized;

    fn to_value(&self) -> Result<Value, ToValueError>
    where
        Self: Sized;
}

#[derive(Debug, Clone, PartialEq, EnumIs, Display, IntoStaticStr)]
pub enum Value {
    Null,
    String(String),
    Bool(bool),
    Number(Number),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}
impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}
impl From<char> for Value {
    fn from(value: char) -> Self {
        Self::String(value.to_string())
    }
}
impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}
impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Self {
        Self::Array(value)
    }
}
impl<T: Into<Value>> FromIterator<(String, T)> for Value {
    fn from_iter<I: IntoIterator<Item = (String, T)>>(iter: I) -> Self {
        Self::Object(iter.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}
impl<T: Into<Value>> FromIterator<T> for Value {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::Array(iter.into_iter().map(|v| v.into()).collect())
    }
}

#[cfg(feature = "toml")]
mod impl_for_toml {
    use crate::value::{Number, Value};
    use toml::Value as TomlValue;

    impl From<Value> for TomlValue {
        fn from(value: Value) -> Self {
            match value {
                Value::String(string) => TomlValue::String(string),
                Value::Bool(bool) => TomlValue::Boolean(bool),
                Value::Array(a) => {
                    let a = a.into_iter().map(|v| v.into()).collect();
                    TomlValue::Array(a)
                }
                Value::Object(v) => {
                    let v = v.into_iter().map(|(k, v)| (k, v.into())).collect();
                    TomlValue::Table(v)
                }
                Value::Null => Value::String("null".to_string()).into(),
                Value::Number(value) => match value {
                    Number::PosInt(int) => TomlValue::Integer(int as i64),
                    Number::SignedInt(int) => TomlValue::Integer(int),
                    Number::Float(float) => TomlValue::Float(float),
                },
            }
        }
    }
    impl From<TomlValue> for Value {
        fn from(value: toml::Value) -> Self {
            match value {
                TomlValue::String(string) => Self::String(string),
                TomlValue::Integer(int) => int.into(),
                TomlValue::Float(float) => float.into(),
                TomlValue::Boolean(boolean) => Self::Bool(boolean),
                TomlValue::Datetime(date) => Self::String(date.to_string()),
                TomlValue::Array(array) => {
                    let array = array.into_iter().map(|v| v.into()).collect();
                    Self::Array(array)
                }
                TomlValue::Table(table) => {
                    let map = table
                        .into_iter()
                        .map(|(key, value)| (key, value.into()))
                        .collect();
                    Self::Object(map)
                }
            }
        }
    }
}
