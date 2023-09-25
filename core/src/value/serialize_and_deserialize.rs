use crate::value::{Number, Value};
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Formatter;

struct ValueVisitor;
macro_rules! visit_num {
    (fn $fn_name:ident<E>(self, v: $ty:ty)) => {
        fn $fn_name<E>(self, v: $ty) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(v.into())
        }
    };
    (fn $fn_name:ident<E>(self, v: $ty:ty) as u64) => {
        fn $fn_name<E>(self, v: $ty) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(v.into())
        }
    };
}
impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Value::Bool(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Value::String(v.to_string()))
    }

    visit_num!(fn visit_i8<E>(self, v: i8));
    visit_num!(fn visit_i16<E>(self, v: i16) );
    visit_num!(fn visit_i32<E>(self, v: i32) );
    visit_num!(fn visit_i64<E>(self, v: i64) );
    visit_num!(fn visit_u8<E>(self, v: u8) );
    visit_num!(fn visit_u16<E>(self, v: u16) );
    visit_num!(fn visit_u32<E>(self, v: u32));
    visit_num!(fn visit_u64<E>(self, v: u64) );
    visit_num!(fn visit_f32<E>(self, v: f32) );
    visit_num!(fn visit_f64<E>(self, v: f64) );

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Value::String(v.to_string()))
    }
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Null)
    }
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Value::String(v))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        Value::deserialize(deserializer)
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut seq = seq;
        let mut values = Vec::new();
        while let Some(value) = seq.next_element::<Value>()? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut map = map;
        let mut values = BTreeMap::new();
        while let Some((key, value)) = map.next_entry::<String, Value>()? {
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}
impl Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Number::PosInt(pos) => serializer.serialize_u64(*pos),

            Number::SignedInt(value) => serializer.serialize_i64(*value),
            Number::Float(f) => serializer.serialize_f64(*f),
        }
    }
}
impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::String(string) => serializer.serialize_str(string),
            Value::Bool(value) => serializer.serialize_bool(*value),

            Value::Array(arr) => arr.serialize(serializer),
            Value::Object(m) => m.serialize(serializer),
            Value::Null => serializer.serialize_none(),
            Value::Number(num) => num.serialize(serializer),
        }
    }
}
