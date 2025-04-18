use std::{
    collections::HashMap,
    fmt::{self, Display, Write},
};

use serde::ser;

#[derive(Debug, Clone, Copy)]
pub enum ArgType {
    Bool,
    Value,
}

/// Support type for automatic CLI argument generation. A CliArgSet is a
/// description of the fields for a given config type, but only the fields
/// that can be effectively loaded from the command like (boolean flags and
/// string or integer options)
#[derive(Debug, Clone)]
pub struct CliArgsSet {
    // Mapping for the fields. The keys of this type are the field name itself,
    // such as `prefix`, and the value includes both information about the
    // field's type and the argument that will be used on the command line,
    // such as `swift-prefix`. Pre-computing these strings makes it easier
    // to reuse them for building a clap parser and for deserialize operations.
    args: HashMap<&'static str, (String, ArgType)>,
}

impl CliArgsSet {
    pub fn iter(&self) -> impl Iterator<Item = ArgSpec<'_>> + '_ {
        self.args
            .iter()
            .map(|(&key, &(ref full_key, arg_type))| ArgSpec {
                key,
                full_key: full_key.as_str(),
                arg_type,
            })
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.args.contains_key(key)
    }
}

pub struct ArgSpec<'a> {
    /// The ID for this argument; this is (what serde thinks is) the field name
    /// in the config; eg, `package_name`
    pub key: &'static str,

    /// The full key as given to clap, including the language prefix and an
    /// underscore to hyphen conversion; eg, `kotlin-package-name`
    pub full_key: &'a str,

    /// The type of argument we think this is, which affects how we configure
    /// the clap parser and how we look up this argument from the parsed cli
    /// arguments
    pub arg_type: ArgType,
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("config container must be a struct type")]
pub struct ArgsSetError;

impl ser::Error for ArgsSetError {
    fn custom<T>(_msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self
    }
}

/// Serializer that constructs a new `CliArgsSet`. It does this by serializing
/// a config object to learn about its fields' names and their types.
pub struct ArgsSetSerializer {
    language: &'static str,
    args: CliArgsSet,
}

impl ArgsSetSerializer {
    pub fn new(language: &'static str) -> Self {
        Self {
            language,
            args: CliArgsSet {
                args: HashMap::new(),
            },
        }
    }
}

impl ser::Serializer for ArgsSetSerializer {
    type Ok = CliArgsSet;
    type Error = ArgsSetError;

    type SerializeSeq = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = ser::Impossible<Self::Ok, Self::Error>;

    type SerializeStruct = Self;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        Err(ArgsSetError)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        Err(ArgsSetError)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(ArgsSetError)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(ArgsSetError)
    }
}

impl ser::SerializeStruct for ArgsSetSerializer {
    type Ok = CliArgsSet;
    type Error = ArgsSetError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        // TODO: duplicate key detection? seems awfully niche
        if let Some(arg_type) = value.serialize(ArgsSetFieldSerializer)? {
            let full_key = format!(
                "{language}-{key}",
                language = self.language,
                key = UnderscoreToHyphen(key)
            );

            self.args.args.insert(key, (full_key, arg_type));
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.args)
    }
}

struct UnderscoreToHyphen<'a>(&'a str);

impl Display for UnderscoreToHyphen<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut chunks = self.0.split('_');

        match chunks.next() {
            None => Ok(()),
            Some(first) => {
                f.write_str(first)?;
                chunks.try_for_each(|chunk| {
                    f.write_char('-')?;
                    f.write_str(chunk)
                })
            }
        }
    }
}

/// This type detects only what type the field has: bool (for a cli flag),
/// primitive (for a cli argument), or anything else, in which case it has
/// no CLI presence.
struct ArgsSetFieldSerializer;

impl ser::Serializer for ArgsSetFieldSerializer {
    type Ok = Option<ArgType>;
    type Error = ArgsSetError;

    // Fill these with no-ops that don't return errors
    type SerializeSeq = NoOpSubSerializer;
    type SerializeTuple = NoOpSubSerializer;
    type SerializeTupleStruct = NoOpSubSerializer;
    type SerializeTupleVariant = NoOpSubSerializer;
    type SerializeMap = NoOpSubSerializer;
    type SerializeStruct = NoOpSubSerializer;
    type SerializeStructVariant = NoOpSubSerializer;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Some(ArgType::Bool))
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Some(ArgType::Value))
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Some(ArgType::Value))
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Some(ArgType::Value))
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Some(ArgType::Value))
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Some(ArgType::Value))
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Some(ArgType::Value))
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(Some(ArgType::Value))
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(Some(ArgType::Value))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(Some(ArgType::Value))
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Some(ArgType::Value))
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Ok(Some(ArgType::Value))
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Some(ArgType::Value))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(Some(ArgType::Value))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Some(ArgType::Value))
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        Ok(Some(ArgType::Value))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(None)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(None)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(Some(ArgType::Value))
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        Ok(None)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(NoOpSubSerializer)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(NoOpSubSerializer)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(NoOpSubSerializer)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(NoOpSubSerializer)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(NoOpSubSerializer)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(NoOpSubSerializer)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(NoOpSubSerializer)
    }
}

struct NoOpSubSerializer;

impl ser::SerializeSeq for NoOpSubSerializer {
    type Ok = Option<ArgType>;
    type Error = ArgsSetError;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(None)
    }
}

impl ser::SerializeTuple for NoOpSubSerializer {
    type Ok = Option<ArgType>;
    type Error = ArgsSetError;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(None)
    }
}

impl ser::SerializeTupleStruct for NoOpSubSerializer {
    type Ok = Option<ArgType>;
    type Error = ArgsSetError;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(None)
    }
}

impl ser::SerializeTupleVariant for NoOpSubSerializer {
    type Ok = Option<ArgType>;
    type Error = ArgsSetError;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(None)
    }
}

impl ser::SerializeMap for NoOpSubSerializer {
    type Ok = Option<ArgType>;
    type Error = ArgsSetError;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        Ok(())
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(None)
    }
}

impl ser::SerializeStruct for NoOpSubSerializer {
    type Ok = Option<ArgType>;
    type Error = ArgsSetError;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(None)
    }
}

impl ser::SerializeStructVariant for NoOpSubSerializer {
    type Ok = Option<ArgType>;
    type Error = ArgsSetError;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(None)
    }
}
