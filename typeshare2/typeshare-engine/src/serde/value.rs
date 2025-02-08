// use serde::{de, ser};

// /// A value representing ANY valid serde data. It isn't totally comprehensive,
// /// but for practical purposes it covers all of our needs.
// pub enum Value {
//     Primitive(Primitive),
//     Collection(Collection),
//     Struct {
//         name: &'static str,
//         data: Composite,
//     },

//     Enum {
//         name: &'static str,
//         variant: &'static str,
//         data: Composite,
//     },
// }

// impl<'de> de::IntoDeserializer<'de> for Value {
//     type Deserializer = Deserializer;

//     fn into_deserializer(self) -> Self::Deserializer {
//         Deserializer(self)
//     }
// }

// pub enum Primitive {
//     Unit,
//     Bool(bool),
//     Signed(i64),
//     Unsigned(u64),
//     Float(f64),
//     Char(char),
//     Option(Option<Box<Value>>),
//     String(String),
//     Bytes(Vec<u8>),
// }

// pub enum Collection {
//     Sequence(Vec<Value>),
//     Map(Vec<KeyValue<Value>>),
// }

// pub enum Composite {
//     Unit,
//     Tuple(Vec<Value>),
//     Newtype(Box<Value>),
//     Struct(Vec<KeyValue<&'static str>>),
// }

// pub struct KeyValue<Key> {
//     pub key: Key,
//     pub value: Value,
// }

// #[derive(Debug, Clone, thiserror::Error)]
// #[error("error while serializing to a serde value; this usually shouldn't happen")]
// struct Error;

// impl ser::Error for Error {
//     fn custom<T>(_msg: T) -> Self
//     where
//         T: std::fmt::Display,
//     {
//         Self
//     }
// }

// pub struct Serializer;

// impl ser::Serializer for Serializer {
//     type Ok = Value;
//     type Error = Error;

//     type SerializeSeq = SerializeCollection<Value>;
//     type SerializeTuple = SerializeCollection<Value>;
//     type SerializeTupleStruct = SerializeStruct<Value>;
//     type SerializeTupleVariant = SerializeEnum<Value>;
//     type SerializeMap = SerializeCollection<KeyValue<Value>>;
//     type SerializeStruct = SerializeStruct<KeyValue<&'static str>>;
//     type SerializeStructVariant = SerializeEnum<KeyValue<&'static str>>;

//     fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
//         Ok(Value::Primitive(Primitive::Bool(v)))
//     }
//     fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
//         self.serialize_i64(v as i64)
//     }

//     fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
//         self.serialize_i64(v as i64)
//     }

//     fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
//         self.serialize_i64(v as i64)
//     }

//     fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
//         Ok(Value::Primitive(Primitive::Signed(v)))
//     }

//     fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
//         self.serialize_u64(v as u64)
//     }

//     fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
//         self.serialize_u64(v as u64)
//     }

//     fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
//         self.serialize_u64(v as u64)
//     }

//     fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
//         Ok(Value::Primitive(Primitive::Unsigned(v)))
//     }

//     fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
//         self.serialize_f64(v as f64)
//     }

//     fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
//         Ok(Value::Primitive(Primitive::Float(v)))
//     }

//     fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
//         Ok(Value::Primitive(Primitive::Char(v)))
//     }

//     fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
//         Ok(Value::Primitive(Primitive::String(v.to_owned())))
//     }

//     fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
//         Ok(Value::Primitive(Primitive::Bytes(v.to_vec())))
//     }

//     fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
//         Ok(Value::Primitive(Primitive::Option(None)))
//     }

//     fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
//     where
//         T: ?Sized + ser::Serialize,
//     {
//         Ok(Value::Primitive(Primitive::Option(Some(Box::new(
//             value.serialize(self)?,
//         )))))
//     }

//     fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
//         Ok(Value::Primitive(Primitive::Unit))
//     }

//     fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
//         Ok(Value::Struct {
//             name,
//             data: Composite::Unit,
//         })
//     }

//     fn serialize_unit_variant(
//         self,
//         name: &'static str,
//         _variant_index: u32,
//         variant: &'static str,
//     ) -> Result<Self::Ok, Self::Error> {
//         Ok(Value::Enum {
//             name,
//             variant,
//             data: Composite::Unit,
//         })
//     }

//     fn serialize_newtype_struct<T>(
//         self,
//         name: &'static str,
//         value: &T,
//     ) -> Result<Self::Ok, Self::Error>
//     where
//         T: ?Sized + ser::Serialize,
//     {
//         Ok(Value::Struct {
//             name,
//             data: Composite::Newtype(Box::new(value.serialize(self)?)),
//         })
//     }

//     fn serialize_newtype_variant<T>(
//         self,
//         name: &'static str,
//         _variant_index: u32,
//         variant: &'static str,
//         value: &T,
//     ) -> Result<Self::Ok, Self::Error>
//     where
//         T: ?Sized + ser::Serialize,
//     {
//         Ok(Value::Enum {
//             name,
//             variant,
//             data: Composite::Newtype(Box::new(value.serialize(self)?)),
//         })
//     }

//     fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
//         Ok(SerializeCollection::new())
//     }

//     fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
//         self.serialize_seq(Some(len))
//     }

//     fn serialize_tuple_struct(
//         self,
//         name: &'static str,
//         _len: usize,
//     ) -> Result<Self::SerializeTupleStruct, Self::Error> {
//         Ok(SerializeStruct::new(name))
//     }

//     fn serialize_tuple_variant(
//         self,
//         name: &'static str,
//         _variant_index: u32,
//         variant: &'static str,
//         _len: usize,
//     ) -> Result<Self::SerializeTupleVariant, Self::Error> {
//         Ok(SerializeEnum::new(name, variant))
//     }

//     fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
//         Ok(SerializeCollection::new())
//     }

//     fn serialize_struct(
//         self,
//         name: &'static str,
//         _len: usize,
//     ) -> Result<Self::SerializeStruct, Self::Error> {
//         Ok(SerializeStruct::new(name))
//     }

//     fn serialize_struct_variant(
//         self,
//         name: &'static str,
//         _variant_index: u32,
//         variant: &'static str,
//         _len: usize,
//     ) -> Result<Self::SerializeStructVariant, Self::Error> {
//         Ok(SerializeEnum::new(name, variant))
//     }
// }

// pub struct SerializeCollection<T> {
//     data: Vec<T>,
// }

// impl<T> SerializeCollection<T> {
//     pub fn new() -> Self {
//         SerializeCollection { data: Vec::new() }
//     }
// }

// impl ser::SerializeSeq for SerializeCollection<Value> {
//     type Ok = Value;
//     type Error = Error;

//     fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
//     where
//         T: ?Sized + ser::Serialize,
//     {
//         self.data.push(value.serialize(Serializer)?);
//         Ok(())
//     }

//     fn end(self) -> Result<Self::Ok, Self::Error> {
//         Ok(Value::Collection(Collection::Sequence(self.data)))
//     }
// }

// impl ser::SerializeTuple for SerializeCollection<Value> {
//     type Ok = Value;
//     type Error = Error;

//     fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
//     where
//         T: ?Sized + ser::Serialize,
//     {
//         self.data.push(value.serialize(Serializer)?);
//         Ok(())
//     }

//     fn end(self) -> Result<Self::Ok, Self::Error> {
//         Ok(Value::Collection(Collection::Sequence(self.data)))
//     }
// }

// impl ser::SerializeMap for SerializeCollection<KeyValue<Value>> {
//     type Ok = Value;
//     type Error = Error;

//     fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
//     where
//         T: ?Sized + ser::Serialize,
//     {
//         self.data.push(KeyValue {
//             key: key.serialize(Serializer)?,
//             value: Value::Primitive(Primitive::Unit),
//         });

//         Ok(())
//     }

//     fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
//     where
//         T: ?Sized + ser::Serialize,
//     {
//         self.data
//             .last_mut()
//             .expect("called serialize_value out of order")
//             .value = value.serialize(Serializer)?;
//         Ok(())
//     }

//     fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
//     where
//         K: ?Sized + ser::Serialize,
//         V: ?Sized + ser::Serialize,
//     {
//         self.data.push(KeyValue {
//             key: key.serialize(Serializer)?,
//             value: value.serialize(Serializer)?,
//         });
//         Ok(())
//     }

//     fn end(self) -> Result<Self::Ok, Self::Error> {
//         Ok(Value::Collection(Collection::Map(self.data)))
//     }
// }

// pub struct SerializeStruct<T> {
//     data: Vec<T>,
//     name: &'static str,
// }

// impl<T> SerializeStruct<T> {
//     pub fn new(name: &'static str) -> Self {
//         SerializeStruct {
//             data: Vec::new(),
//             name,
//         }
//     }
// }

// impl ser::SerializeTupleStruct for SerializeStruct<Value> {
//     type Ok = Value;
//     type Error = Error;

//     fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
//     where
//         T: ?Sized + ser::Serialize,
//     {
//         self.data.push(value.serialize(Serializer)?);
//         Ok(())
//     }

//     fn end(self) -> Result<Self::Ok, Self::Error> {
//         Ok(Value::Struct {
//             name: self.name,
//             data: Composite::Tuple(self.data),
//         })
//     }
// }

// impl ser::SerializeStruct for SerializeStruct<KeyValue<&'static str>> {
//     type Ok = Value;
//     type Error = Error;

//     fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
//     where
//         T: ?Sized + ser::Serialize,
//     {
//         self.data.push(KeyValue {
//             key,
//             value: value.serialize(Serializer)?,
//         });

//         Ok(())
//     }

//     fn end(self) -> Result<Self::Ok, Self::Error> {
//         Ok(Value::Struct {
//             name: self.name,
//             data: Composite::Struct(self.data),
//         })
//     }
// }

// pub struct SerializeEnum<T> {
//     name: &'static str,
//     variant: &'static str,
//     data: Vec<T>,
// }

// impl<T> SerializeEnum<T> {
//     pub fn new(name: &'static str, variant: &'static str) -> Self {
//         SerializeEnum {
//             name,
//             variant,
//             data: Vec::new(),
//         }
//     }
// }

// impl ser::SerializeTupleVariant for SerializeEnum<Value> {
//     type Ok = Value;
//     type Error = Error;

//     fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
//     where
//         T: ?Sized + ser::Serialize,
//     {
//         self.data.push(value.serialize(Serializer)?);
//         Ok(())
//     }

//     fn end(self) -> Result<Self::Ok, Self::Error> {
//         Ok(Value::Enum {
//             name: self.name,
//             variant: self.variant,
//             data: Composite::Tuple(self.data),
//         })
//     }
// }

// impl ser::SerializeStructVariant for SerializeEnum<KeyValue<&'static str>> {
//     type Ok = Value;
//     type Error = Error;

//     fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
//     where
//         T: ?Sized + ser::Serialize,
//     {
//         self.data.push(KeyValue {
//             key,
//             value: value.serialize(Serializer)?,
//         });
//         Ok(())
//     }

//     fn end(self) -> Result<Self::Ok, Self::Error> {
//         Ok(Value::Enum {
//             name: self.name,
//             variant: self.variant,
//             data: Composite::Struct(self.data),
//         })
//     }
// }

// pub struct Deserializer(pub Value);

// impl<'de> de::Deserializer<'de> for Deserializer {
//     type Error;

//     fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         match self.0 {
//             Value::Primitive(primitive) => match primitive {
//                 Primitive::Unit => visitor.visit_unit(),
//                 Primitive::Bool(v) => visitor.visit_bool(v),
//                 Primitive::Signed(v) => visitor.visit_i64(v),
//                 Primitive::Unsigned(v) => visitor.visit_u64(v),
//                 Primitive::Float(v) => visitor.visit_f64(v),
//                 Primitive::Char(v) => visitor.visit_char(v),
//                 Primitive::Option(Some(v)) => visitor.visit_some(Deserializer(*v)),
//                 Primitive::Option(None) => visitor.visit_none(),
//                 Primitive::String(v) => visitor.visit_string(v),
//                 Primitive::Bytes(v) => visitor.visit_byte_buf(v),
//             },
//             Value::Collection(collection) => match collection {
//                 Collection::Sequence(values) => visitor.visit_seq(de::value::),
//                 Collection::Map(key_values) => visitor.visit_map(Access(key_values.into_iter())),
//             },
//             Value::Struct { name, data } => todo!(),
//             Value::Enum {
//                 name,
//                 variant,
//                 data,
//             } => todo!(),
//         }
//     }

//     serde::forward_to_deserialize_any! {
//         bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
//         bytes byte_buf option unit unit_struct newtype_struct seq tuple
//         tuple_struct map struct enum identifier ignored_any
//     }
// }

// pub struct SeqAccess<I> {
//     iter: I,
// }

// impl<'de, I> de::SeqAccess<'de> for SeqAccess<I>
// where
//     I: Iterator<Item = Value>,
// {
//     type Error;

//     fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
//     where
//         T: de::DeserializeSeed<'de>,
//     {
//         self.iter
//             .next()
//             .map(|item| seed.deserialize(Deserializer(item)))
//             .transpose()
//     }
// }

// pub struct MapAccess<I> {
//     iter: I,
//     saved_value: Option<Value>,
// }

// impl<'de, I> de::MapAccess<'de> for MapAccess<I>
// where
//     I: Iterator<Item = KeyValue<Value>>,
// {
//     type Error;

//     fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
//     where
//         K: de::DeserializeSeed<'de>,
//     {
//         self.iter.next().map(|key_value| {
//             self.saved_value = Some(key_value.value);
//             seed.deserialize(Deserializer(key))
//         })
//     }

//     fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::DeserializeSeed<'de>,
//     {
//         seed.deserialize(Deserializer())
//     }
// }
