use crate::value::{ToValueError, Value};
use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};
use serde::{Serialize, Serializer};
use std::collections::BTreeMap;

macro_rules! serialize_by_into {
    ($fun:ident, $t:ty) => {
        fn $fun(self, v: $t) -> Result<Self::Ok, Self::Error> {
            Ok(v.into())
        }
    };
}
impl SerializeSeq for Value {
    type Ok = Value;
    type Error = ToValueError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        match self {
            Value::Array(array) => {
                let value = value.serialize(ValueSerializer)?;
                array.push(value);
                Ok(())
            }
            _ => panic!("Value::serialize_element called on non-array value"),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self)
    }
}
impl SerializeTuple for Value {
    type Ok = Value;
    type Error = ToValueError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        match self {
            Value::Array(array) => {
                let value = value.serialize(ValueSerializer)?;
                array.push(value);
                Ok(())
            }
            _ => panic!("Value::serialize_element called on non-array value"),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self)
    }
}
impl SerializeTupleStruct for Value {
    type Ok = Value;
    type Error = ToValueError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        match self {
            Value::Array(array) => {
                let value = value.serialize(ValueSerializer)?;
                array.push(value);
                Ok(())
            }
            _ => panic!("Value::serialize_element called on non-array value"),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self)
    }
}

pub struct MapSerializer {
    map: BTreeMap<String, Value>,
    key: Option<String>,
}
impl SerializeMap for MapSerializer {
    type Ok = Value;
    type Error = ToValueError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let key = key.serialize(ValueSerializer)?;

        match key {
            Value::String(key) => {
                if let Some(value) = self.key.take() {
                    self.map.insert(value, Value::Null);
                } else {
                    self.key = Some(key);
                }
                Ok(())
            }
            _ => Err(ToValueError::KeyMustBeString),
        }
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let value = value.serialize(ValueSerializer)?;

        if let Some(key) = self.key.take() {
            self.map.insert(key, value);
            Ok(())
        } else {
            Err(ToValueError::KeyNotPresent)
        }
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        if let Some(key) = self.key.take() {
            self.map.insert(key, Value::Null);
        }
        Ok(Value::Object(self.map))
    }
}
impl SerializeStruct for MapSerializer {
    type Ok = Value;
    type Error = ToValueError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let value = value.serialize(ValueSerializer)?;
        self.map.insert(key.to_string(), value);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Object(self.map))
    }
}

pub struct ValueStructVariant {
    pub variant: String,
    pub value: BTreeMap<String, Value>,
}
impl SerializeStructVariant for ValueStructVariant {
    type Ok = Value;
    type Error = ToValueError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let value = value.serialize(ValueSerializer)?;
        self.value.insert(key.to_string(), value);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut map = BTreeMap::new();
        map.insert(self.variant, Value::Object(self.value));
        Ok(Value::Object(map))
    }
}
pub struct ValueTupleVariant {
    pub variant: String,
    pub value: Vec<Value>,
}
impl SerializeTupleVariant for ValueTupleVariant {
    type Ok = Value;
    type Error = ToValueError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let value = value.serialize(ValueSerializer)?;
        self.value.push(value);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut map = BTreeMap::new();
        map.insert(self.variant, Value::Array(self.value));
        Ok(Value::Object(map))
    }
}

pub struct ValueSerializer;
impl Serializer for ValueSerializer {
    type Ok = Value;
    type Error = ToValueError;
    type SerializeSeq = Value;
    type SerializeTuple = Value;
    type SerializeTupleStruct = Value;
    type SerializeTupleVariant = ValueTupleVariant;
    type SerializeMap = MapSerializer;
    type SerializeStruct = MapSerializer;
    type SerializeStructVariant = ValueStructVariant;

    serialize_by_into!(serialize_bool, bool);
    serialize_by_into!(serialize_i8, i8);
    serialize_by_into!(serialize_i16, i16);
    serialize_by_into!(serialize_i32, i32);
    serialize_by_into!(serialize_i64, i64);
    serialize_by_into!(serialize_u8, u8);
    serialize_by_into!(serialize_u16, u16);
    serialize_by_into!(serialize_u32, u32);
    serialize_by_into!(serialize_u64, u64);
    serialize_by_into!(serialize_f32, f32);
    serialize_by_into!(serialize_f64, f64);
    serialize_by_into!(serialize_str, &str);
    serialize_by_into!(serialize_char, char);

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut array = Vec::with_capacity(v.len());
        for byte in v.iter() {
            array.push((*byte).into());
        }
        Ok(Value::Array(array))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        let mut map = BTreeMap::new();
        map.insert(variant.to_string(), Value::Null);
        Ok(Value::Object(map))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        let mut map = BTreeMap::new();
        map.insert(variant.to_string(), value.serialize(self)?);
        Ok(Value::Object(map))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(Value::Array(Vec::with_capacity(len.unwrap_or(0))))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(Value::Array(Vec::with_capacity(len)))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(Value::Array(Vec::with_capacity(len)))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(ValueTupleVariant {
            variant: variant.to_string(),
            value: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(MapSerializer {
            map: BTreeMap::new(),
            key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(MapSerializer {
            map: BTreeMap::new(),
            key: None,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(ValueStructVariant {
            variant: variant.to_string(),
            value: BTreeMap::new(),
        })
    }
}
