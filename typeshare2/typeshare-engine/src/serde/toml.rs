use std::marker::PhantomData;

use serde::{de, ser};

/// Borrowing deserializer for a `toml::Value`
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

#[derive(Debug, thiserror::Error)]
pub enum SerializeError {
    #[error("error from Serialize type: {0}")]
    Custom(String),

    #[error("tried to serialize something other than a struct or a map to a toml table")]
    NotStruct,

    #[error(
        "tried to serialize a map to a toml table. This will be supported \
        in the future; file an issue if you need this for your use case"
    )]
    Map,

    #[error("tried to serialize an integer that was out of range for an i64")]
    IntOutOfRange,

    #[error("tried to serialize bytes; toml can't carry raw binary data")]
    Bytes,
}

impl ser::Error for SerializeError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom(msg.to_string())
    }
}

/// Serializer that turns something directly into a toml table
pub struct TableSerializer;

impl ser::Serializer for TableSerializer {
    type Ok = toml::Table;
    type Error = SerializeError;

    type SerializeSeq = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = ser::Impossible<Self::Ok, Self::Error>;

    type SerializeStruct = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        Err(SerializeError::NotStruct)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        Err(SerializeError::NotStruct)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(SerializeError::NotStruct)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(SerializeError::Map)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(SerializeError::NotStruct)
    }
}

impl ser::SerializeStruct for TableSerializer {
    type Ok = ();
    type Error = SerializeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// Serializer that adds a value to a table
pub struct FieldSerializer<'a> {
    key: &'static str,
    table: &'a mut toml::Table,
}

impl FieldSerializer<'_> {
    pub fn serialize(self, value: toml::Value) -> Result<(), SerializeError> {
        self.table.insert(self.key.to_owned(), value);
        Ok(())
    }
}

impl ser::Serializer for FieldSerializer<'_> {
    type Ok = ();
    type Error = SerializeError;

    type SerializeSeq;
    type SerializeTuple;
    type SerializeTupleStruct;
    type SerializeTupleVariant;
    type SerializeMap;
    type SerializeStruct;
    type SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.serialize(toml::Value::Boolean(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v.into())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v.into())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v.into())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.serialize(toml::Value::Integer(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v.into())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v.into())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v.into())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v.try_into().map_err(|_| SerializeError::IntOutOfRange)?)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v.into())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.serialize(toml::Value::Float(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let mut buffer = [0; 4];
        let s = v.encode_utf8(&mut buffer);
        self.serialize_str(s)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.serialize(toml::Value::String(v.to_owned()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::Bytes)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        todo!()
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}
