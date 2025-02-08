pub mod args;
pub mod config;
pub mod toml;

use serde::de;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
#[error("error from deserialized type: {0}")]
pub struct SimpleError(String);

impl de::Error for SimpleError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self(msg.to_string())
    }
}

/// Deserializer that always produces an empty map. Used as a clever way to
/// default-initialize language configs.
pub struct EmptyDeserializer;

impl<'de> de::Deserializer<'de> for EmptyDeserializer {
    type Error = SimpleError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> de::MapAccess<'de> for EmptyDeserializer {
    type Error = SimpleError;

    fn next_key_seed<K>(&mut self, _seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        Ok(None)
    }

    fn next_value_seed<V>(&mut self, _seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        panic!("EmptyDeserializer never produces a value")
    }
}
