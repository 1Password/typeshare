use super::Value;
use crate::value::{FromValueError, Number};
use serde::de::{
    DeserializeSeed, EnumAccess, Error, IntoDeserializer, MapAccess, SeqAccess, Unexpected,
    VariantAccess, Visitor,
};
use serde::{forward_to_deserialize_any, Deserialize, Deserializer};
use std::collections::BTreeMap;

use std::vec::IntoIter;

struct EnumDeserializer {
    variant: String,
    value: Option<Value>,
}
struct VariantDeserializer {
    value: Option<Value>,
}
impl<'de> VariantAccess<'de> for VariantDeserializer {
    type Error = FromValueError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.value {
            Some(value) => Deserialize::deserialize(value),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Array(v)) => {
                if v.is_empty() {
                    visitor.visit_unit()
                } else {
                    visit_array(v, visitor)
                }
            }
            Some(other) => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"tuple variant",
            )),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Object(v)) => visit_object(v, visitor),

            Some(other) => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"struct variant",
            )),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}
impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = FromValueError;
    type Variant = VariantDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), FromValueError>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = self.variant.into_deserializer();
        let visitor = VariantDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

pub struct MapKeyDeserializer {
    key: String,
}
impl<'de> Deserializer<'de> for MapKeyDeserializer {
    type Error = FromValueError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.key)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
pub struct MapDeserializer {
    iter: <BTreeMap<String, Value> as IntoIterator>::IntoIter,
    value: Option<Value>,
}
impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = FromValueError;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, FromValueError>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                let key_de = MapKeyDeserializer { key };
                seed.deserialize(key_de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, FromValueError>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}
pub struct ArrayDeserializer(IntoIter<Value>);
impl<'de> SeqAccess<'de> for ArrayDeserializer {
    type Error = FromValueError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let Self(iter) = self;
        match iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }
    fn size_hint(&self) -> Option<usize> {
        self.0.size_hint().1
    }
}

macro_rules! deserialize_num {
    ($de_name:ident, $num_v:ident, $expected:literal, $visit_fn:ident as $as_t:ty) => {
        fn $de_name<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            if let Value::Number(Number::$num_v(v)) = self {
                visitor.$visit_fn(v as $as_t)
            } else {
                Err(FromValueError::custom($expected))
            }
        }
    };
}
fn visit_array<'de, V>(v: Vec<Value>, visitor: V) -> Result<V::Value, FromValueError>
where
    V: Visitor<'de>,
{
    let array = ArrayDeserializer(v.into_iter());
    visitor.visit_seq(array)
}

fn visit_object<'de, V>(v: BTreeMap<String, Value>, visitor: V) -> Result<V::Value, FromValueError>
where
    V: Visitor<'de>,
{
    let object = MapDeserializer {
        iter: v.into_iter(),
        value: None,
    };
    visitor.visit_map(object)
}
impl<'de> Deserializer<'de> for Value {
    type Error = FromValueError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_none(),
            Value::String(v) => visitor.visit_string(v),
            Value::Bool(v) => visitor.visit_bool(v),
            Value::Number(v) => match v {
                Number::PosInt(v) => visitor.visit_u64(v),
                Number::SignedInt(v) => visitor.visit_i64(v),
                Number::Float(v) => visitor.visit_f64(v),
            },
            Value::Array(v) => visit_array(v, visitor),
            Value::Object(v) => visit_object(v, visitor),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Bool(v) = self {
            visitor.visit_bool(v)
        } else {
            Err(FromValueError::custom("Expected a bool"))
        }
    }

    deserialize_num!(deserialize_i8, SignedInt, "Expected a i8", visit_i8 as i8);
    deserialize_num!(
        deserialize_i16,
        SignedInt,
        "Expected a i16",
        visit_i16 as i16
    );
    deserialize_num!(
        deserialize_i32,
        SignedInt,
        "Expected a i32",
        visit_i32 as i32
    );
    deserialize_num!(
        deserialize_i64,
        SignedInt,
        "Expected a i64",
        visit_i64 as i64
    );
    deserialize_num!(deserialize_u8, PosInt, "Expected a u8", visit_u8 as u8);
    deserialize_num!(deserialize_u16, PosInt, "Expected a u16", visit_u16 as u16);
    deserialize_num!(deserialize_u32, PosInt, "Expected a u32", visit_u32 as u32);
    deserialize_num!(deserialize_u64, PosInt, "Expected a u64", visit_u64 as u64);
    deserialize_num!(deserialize_f32, Float, "Expected a f32", visit_f32 as f32);
    deserialize_num!(deserialize_f64, Float, "Expected a f64", visit_f64 as f64);

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::String(v) = self {
            if v.len() == 1 {
                visitor.visit_char(v.chars().next().unwrap())
            } else {
                Err(FromValueError::custom("Expected a char"))
            }
        } else {
            Err(FromValueError::custom("Expected a char"))
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::String(v) = self {
            visitor.visit_str(&v)
        } else {
            Err(FromValueError::custom("Expected a string"))
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::String(v) = self {
            visitor.visit_string(v)
        } else {
            Err(FromValueError::custom("Expected a string"))
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::String(v) => visitor.visit_bytes(v.as_bytes()),
            Value::Array(v) => {
                let array = ArrayDeserializer(v.into_iter());
                visitor.visit_seq(array)
            }
            v => Err(FromValueError::custom(format!(
                "Expected a string or array, got {:?}",
                v
            ))),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self == Value::Null {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_unit(),
            v => Err(FromValueError::invalid_type(
                Unexpected::Other(v.into()),
                &"null",
            )),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Array(v) = self {
            visit_array(v, visitor)
        } else {
            Err(FromValueError::custom("Expected an array"))
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Object(v) = self {
            visit_object(v, visitor)
        } else {
            Err(FromValueError::custom("Expected an object"))
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Array(v) => visit_array(v, visitor),
            Value::Object(v) => visit_object(v, visitor),
            e => Err(FromValueError::invalid_type(
                Unexpected::Other(e.into()),
                &"array or object",
            )),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (variant, value) = match self {
            Value::Object(value) => {
                if value.len() != 1 {
                    return Err(serde::de::Error::invalid_value(
                        Unexpected::Map,
                        &"map with a single key",
                    ));
                }
                let mut iter = value.into_iter();
                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => {
                        return Err(serde::de::Error::invalid_value(
                            Unexpected::Map,
                            &"map with a single key",
                        ));
                    }
                };
                (variant, Some(value))
            }
            Value::String(variant) => (variant, None),
            other => {
                return Err(serde::de::Error::invalid_type(
                    Unexpected::Other(other.into()),
                    &"string or map",
                ));
            }
        };

        visitor.visit_enum(EnumDeserializer { variant, value })
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }
}
