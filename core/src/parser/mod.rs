use crate::parsed_types::{ParsedData, Source};
use serde::de::DeserializeOwned;
use std::error::Error;

pub trait Parser {
    type Error: Error;
    type Config: DeserializeOwned;
    fn file_type() -> &'static str
    where
        Self: Sized;
    fn parser_name() -> &'static str
    where
        Self: Sized;

    fn file_extensions() -> Vec<&'static str>
    where
        Self: Sized;

    fn parse_from_str<I: AsRef<str>>(
        &self,
        content: I,
        source: Source,
    ) -> Result<ParsedData, Self::Error>
    where
        Self: Sized,
    {
        let mut parsed_data = ParsedData::default();
        self.parse_into_from_str(content, &mut parsed_data, source)?;
        Ok(parsed_data)
    }
    fn parse_into_from_str<I: AsRef<str>>(
        &self,
        input: I,
        parsed_data: &mut ParsedData,
        source: Source,
    ) -> Result<(), Self::Error>
    where
        Self: Sized;
}
