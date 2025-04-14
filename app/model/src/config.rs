/*!
Utilities for configuration for Language implementations
 */

use serde::{de, ser};

/// Languages that take no configuration can use this as their
/// `Language::Config` type
#[derive(Debug, Clone, Copy, Default)]
pub struct Unconfigured;

struct DeadKey;

impl<'de> de::Deserialize<'de> for DeadKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_identifier({
            struct Visitor;

            impl<'de> de::Visitor<'de> for Visitor {
                type Value = DeadKey;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("a struct key")
                }

                fn visit_str<E>(self, _v: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(DeadKey)
                }

                fn visit_bytes<E>(self, _v: &[u8]) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(DeadKey)
                }
            }

            Visitor
        })
    }
}

impl ser::Serialize for Unconfigured {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use ser::SerializeStruct as _;

        serializer.serialize_struct("Unconfigured", 0)?.end()
    }
}

impl<'de> serde::Deserialize<'de> for Unconfigured {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct("Unconfigured", &[], {
            struct Visitor;

            impl<'de> de::Visitor<'de> for Visitor {
                type Value = Unconfigured;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, "an empty struct")
                }

                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: de::MapAccess<'de>,
                {
                    while let Some((DeadKey, de::IgnoredAny)) = map.next_entry()? {}
                    Ok(Unconfigured)
                }
            }

            Visitor
        })
    }
}
