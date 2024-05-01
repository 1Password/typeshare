use chrono::{DateTime, SecondsFormat, Utc};
use serde::de::{Error, Unexpected, Visitor};
use serde::{Deserializer, Serializer};
use std::fmt;

struct DateTimeVisitor;

pub fn serialize<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(dt.to_rfc3339_opts(SecondsFormat::Millis, true).as_str())
}

pub fn deserialize<'de, D>(d: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_str(DateTimeVisitor)
}

impl<'de> Visitor<'de> for DateTimeVisitor {
    type Value = DateTime<Utc>;

    fn expecting(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "date and time in ISO 8601")
    }

    /// Deserialize a ISO 8601 formatted time
    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<DateTime<Utc>, E>
    where
        E: Error,
    {
        Ok(DateTime::parse_from_rfc3339(value)
            .map_err(|_| Error::invalid_value(Unexpected::Str(value), &self))?
            .with_timezone(&Utc))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Test {
        #[serde(with = "super")]
        time: DateTime<Utc>,
    }

    #[test]
    fn test_deserialize() {
        let data = r#"{"time":"2020-02-01T03:16:46.229Z"}"#;
        let expect = Test {
            time: DateTime::from_timestamp(1_580_527_006, 229_000_000).unwrap(),
        };

        assert_eq!(serde_json::from_str::<Test>(data).unwrap(), expect);
    }

    #[test]
    fn test_serialize() {
        let data = Test {
            time: DateTime::from_timestamp(1_580_527_006, 229_000_000).unwrap(),
        };
        let expect = r#"{"time":"2020-02-01T03:16:46.229Z"}"#;

        assert_eq!(serde_json::to_string(&data).unwrap(), expect);
    }
}
