use std::{
    ffi::{OsStr, OsString},
    str::FromStr,
};

use itertools::Itertools;
use serde::de::{self, value::BorrowedStrDeserializer};

use super::args::{ArgSpec, ArgType, CliArgsSet};

#[derive(Debug, thiserror::Error)]
pub enum ConfigDeserializeError {
    #[error("error from Deserialize type: {0}")]
    Custom(String),

    #[error("command line argument value wasn't valid UTF-8: {0:?}")]
    NonUtf8CliArgument(OsString),

    #[error("error parsing command line value {0:?}")]
    ParseError(
        String,
        #[source] Box<dyn std::error::Error + Send + Sync + 'static>,
    ),
}

impl de::Error for ConfigDeserializeError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom(msg.to_string())
    }
}

/// Deserializer type that combines the values in `config`, which come from a
/// config file, with the values in `args`, which come from the CLI.
pub struct ConfigDeserializer<'a, 'config> {
    config: &'config toml::Table,
    args: &'config clap::ArgMatches,
    spec: &'a CliArgsSet,
}

impl<'a, 'config> ConfigDeserializer<'a, 'config> {
    pub fn new(
        config: &'config toml::Table,
        args: &'config clap::ArgMatches,
        spec: &'a CliArgsSet,
    ) -> Self {
        Self { config, args, spec }
    }
}

impl<'a: 'de, 'de> de::Deserializer<'de> for ConfigDeserializer<'a, 'de> {
    type Error = ConfigDeserializeError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(ConfigMapAccess {
            state: ConfigMapState::CliValues {
                iterator: self.spec.iter(),
                config: self.config,
                args: self.args,
            },
            spec: self.spec,
            saved_value: None,
        })
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

enum SavedValue<'a> {
    True,
    CliArg(&'a OsStr),
    TomlValue(&'a toml::Value),
}

enum ConfigMapState<'config, ArgsIter> {
    CliValues {
        iterator: ArgsIter,

        config: &'config toml::Table,
        args: &'config clap::ArgMatches,
    },
    TomlValues {
        iterator: toml::map::Iter<'config>,
    },
}

struct ConfigMapAccess<'a, 'config, ArgsIter> {
    state: ConfigMapState<'config, ArgsIter>,
    spec: &'a CliArgsSet,

    saved_value: Option<SavedValue<'config>>,
}

impl<'de, ArgsIter> de::MapAccess<'de> for ConfigMapAccess<'_, 'de, ArgsIter>
where
    ArgsIter: Iterator<Item = ArgSpec<'de>>,
{
    type Error = ConfigDeserializeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        // Basic algorithm:
        //
        // - Loop over the args spec. For each key, look it up in the matches,
        //   and fall back to the config if it's absent.
        // - Loop over the config, skipping keys that were present in the
        //   args spec.
        loop {
            return match self.state {
                ConfigMapState::CliValues {
                    ref mut iterator,
                    config,
                    args,
                } => {
                    // Get the next argument that we believe might come from
                    // the command line. If there is None, we move on to
                    // parsing all remaining values from the config file.
                    let Some(arg_spec) = iterator.next() else {
                        self.state = ConfigMapState::TomlValues {
                            iterator: config.iter(),
                        };
                        continue;
                    };

                    // First, try to get the value from the command line. The
                    // arg_spec tells us if the command line was supposed to
                    // contain a flag (with no argument) or an argument.
                    let value = match arg_spec.arg_type {
                        ArgType::Bool => {
                            args.get_flag(arg_spec.full_key).then_some(SavedValue::True)
                        }

                        ArgType::Value => args
                            .try_get_raw(arg_spec.full_key)
                            .unwrap_or_else(|_| {
                                panic!(
                                    "argument --{} wasn't recognized by clap; \
                                    this is probably a typeshare bug",
                                    arg_spec.full_key
                                )
                            })
                            .map(|values| {
                                values.exactly_one().unwrap_or_else(|_| {
                                    panic!(
                                        "More than one argument given for --{}; \
                                        clap should have prevented this",
                                        arg_spec.full_key
                                    )
                                })
                            })
                            .map(SavedValue::CliArg),
                    };

                    // If no value was present on the command line for this arg,
                    // fall back to the command line
                    let value =
                        value.or_else(|| config.get(arg_spec.key).map(SavedValue::TomlValue));

                    // If no value was present either on the command line or
                    // in the config file, we skip this key entirely
                    let Some(value) = value else {
                        continue;
                    };

                    // Got a value!
                    self.saved_value = Some(value);
                    seed.deserialize(BorrowedStrDeserializer::new(arg_spec.key))
                        .map(Some)
                }
                ConfigMapState::TomlValues { ref mut iterator } => {
                    // This part is much simpler: iterate over the config file,
                    // skipping fields that match those found in the argspec.
                    // Once this iterator is exhausted, the deserializing is
                    // done!
                    iterator
                        .find(|(key, _)| !self.spec.contains_key(key))
                        .map(|(key, value)| {
                            self.saved_value = Some(SavedValue::TomlValue(value));
                            seed.deserialize(BorrowedStrDeserializer::new(key))
                        })
                        .transpose()
                }
            };
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        // We definitely deserialized a key, which means we saved a value to
        // deserialize.
        match self
            .saved_value
            .take()
            .expect("called next_value_seed out of order")
        {
            SavedValue::True => seed.deserialize(de::value::BoolDeserializer::new(true)),
            SavedValue::CliArg(os_str) => {
                seed.deserialize(CliArgumentDeserializer { value: os_str })
            }
            SavedValue::TomlValue(value) => {
                seed.deserialize(super::toml::ValueDeserializer::new(value))
            }
        }
    }
}

/// Deserializer for a value we got from the command line. Generally handles
/// FromStr for this value.
struct CliArgumentDeserializer<'a> {
    value: &'a OsStr,
}

impl<'a> CliArgumentDeserializer<'a> {
    pub fn get_bytes(&self) -> &'a [u8] {
        self.value.as_encoded_bytes()
    }

    pub fn get_str(&self) -> Result<&'a str, ConfigDeserializeError> {
        self.value
            .to_str()
            .ok_or_else(|| ConfigDeserializeError::NonUtf8CliArgument(self.value.to_owned()))
    }

    pub fn parse<T: FromStr>(&self) -> Result<T, ConfigDeserializeError>
    where
        T::Err: std::error::Error + Send + Sync + 'static,
    {
        let s = self.get_str()?;

        s.parse()
            .map_err(|err| ConfigDeserializeError::ParseError(s.to_owned(), Box::new(err)))
    }
}

impl<'de> de::Deserializer<'de> for CliArgumentDeserializer<'de> {
    type Error = ConfigDeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(self.get_bytes())
    }

    serde::forward_to_deserialize_any! {
        bool bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct ignored_any
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i8(self.parse()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i16(self.parse()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i32(self.parse()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i64(self.parse()?)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i128(self.parse()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u8(self.parse()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u16(self.parse()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u32(self.parse()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u64(self.parse()?)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u128(self.parse()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f32(self.parse()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f64(self.parse()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.get_str()?)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.get_str()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.get_str()?)
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

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        // BorrowedStrDeserializer elegantly handles string -> unit enum
        // conversions
        de::value::BorrowedStrDeserializer::new(self.get_str()?)
            .deserialize_enum(name, variants, visitor)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.get_str()?)
    }
}
