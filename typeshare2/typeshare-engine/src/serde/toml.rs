//! Borrowing deserializer for a toml value

use std::marker::PhantomData;

use serde::de;
use std::io::Write;

pub struct ValueDeserializer<'a, E> {
    value: &'a toml::Value,
    err: PhantomData<E>,
}

impl<'a, E> ValueDeserializer<'a, E> {
    pub fn new(value: &'a toml::Value) -> Self {
        ValueDeserializer {
            value,
            err: PhantomData,
        }
    }
}

impl<'de, E: de::Error> de::Deserializer<'de> for ValueDeserializer<'de, E> {
    type Error = E;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match *self.value {
            toml::Value::String(ref s) => visitor.visit_borrowed_str(s),
            toml::Value::Integer(i) => visitor.visit_i64(i),
            toml::Value::Float(f) => visitor.visit_f64(f),
            toml::Value::Boolean(b) => visitor.visit_bool(b),
            // In the future we'd like to more correctly support datetimes;
            // toml includes some undocumented fancy trickery to pass them in a
            // structured way
            toml::Value::Datetime(date) => {
                // TODO: I'm pretty sure the largest possible string is like 30
                // characters, so we could write to a local [u8; 64]. The trouble
                // is that there's no clean way to turn that into an &str
                // without extra UTF8 checks or unsafe.
                let date = date.to_string();
                visitor.visit_string(date)
            }
            toml::Value::Array(ref values) => visitor.visit_seq(SeqAccess::new(values)),
            toml::Value::Table(ref map) => visitor.visit_map(MapAccess::new(
                map.iter().map(|(key, value)| (key.as_str(), value)),
            )),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    // TODO: support for deserializing enums. `toml` does the same thing that
    // JSON does, where it serializes them as `{"Variant": ...}`
}
struct SeqAccess<'a, E> {
    values: &'a [toml::Value],
    err: PhantomData<E>,
}
impl<'a, E> SeqAccess<'a, E> {
    pub fn new(values: &'a [toml::Value]) -> Self {
        SeqAccess {
            values,
            err: PhantomData,
        }
    }
}

impl<'de, E: de::Error> de::SeqAccess<'de> for SeqAccess<'de, E> {
    type Error = E;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        self.values
            .split_first()
            .map(|(value, tail)| {
                self.values = tail;
                seed.deserialize(ValueDeserializer::new(value))
            })
            .transpose()
    }
}

struct MapAccess<'a, I, E> {
    values: I,
    saved_value: Option<&'a toml::Value>,
    err: PhantomData<E>,
}

impl<'a, I, E> MapAccess<'a, I, E>
where
    I: Iterator<Item = (&'a str, &'a toml::Value)>,
{
    pub fn new(values: I) -> Self {
        MapAccess {
            values,
            saved_value: None,
            err: PhantomData,
        }
    }
}

impl<'de, I, E: de::Error> de::MapAccess<'de> for MapAccess<'de, I, E>
where
    I: Iterator<Item = (&'de str, &'de toml::Value)>,
{
    type Error = E;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        self.values
            .next()
            .map(|(key, value)| {
                self.saved_value = Some(value);
                seed.deserialize(de::value::BorrowedStrDeserializer::new(key))
            })
            .transpose()
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(ValueDeserializer::new(
            self.saved_value
                .take()
                .expect("called next_value_seed out of order"),
        ))
    }

    fn next_entry_seed<K, V>(
        &mut self,
        kseed: K,
        vseed: V,
    ) -> Result<Option<(K::Value, V::Value)>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
        V: de::DeserializeSeed<'de>,
    {
        self.values
            .next()
            .map(|(key, value)| {
                let key = kseed.deserialize(de::value::BorrowedStrDeserializer::new(key))?;
                let value = vseed.deserialize(ValueDeserializer::new(value))?;

                Ok((key, value))
            })
            .transpose()
    }
}
