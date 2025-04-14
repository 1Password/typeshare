/*!
Utilities for configuration for Language implementations
 */

use serde::{de, ser::SerializeStruct};

/// Languages that take no configuration can use this as their
/// `Language::Config` type
#[derive(Debug, Clone, Copy, Default)]
pub struct Unconfigured;

impl serde::Serialize for Unconfigured {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_struct("Unconfigured", 0)?.end()
    }
}

impl<'de> serde::Deserialize<'de> for Unconfigured {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
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
                while let Some((de::IgnoredAny, de::IgnoredAny)) = map.next_entry()? {}
                Ok(Unconfigured)
            }
        }
        deserializer.deserialize_struct("Unconfigured", &[], Visitor)
    }
}
