use crate::ffi_interop::ffi_v1::{FFIArray, FFIType};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

#[derive(Default, Clone, PartialEq)]
#[repr(transparent)]
pub struct FFIString(FFIArray<u8>);
impl FFIString {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
impl Debug for FFIString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = <Self as AsRef<str>>::as_ref(self);
        Debug::fmt(string, f)
    }
}

impl Serialize for FFIString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string = <Self as AsRef<str>>::as_ref(self);
        serializer.serialize_str(string.as_ref())
    }
}
impl<'de> Deserialize<'de> for FFIString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Ok(Self::from(string))
    }
}

impl Display for FFIString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", AsRef::<str>::as_ref(self))
    }
}
impl AsRef<str> for FFIString {
    fn as_ref(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(AsRef::<[u8]>::as_ref(&self.0)) }
    }
}
impl Deref for FFIString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
impl From<String> for FFIString {
    fn from(value: String) -> Self {
        let bytes = value.into_bytes();
        Self(bytes.try_into().unwrap())
    }
}

impl FFIType for FFIString {
    type SafeType = String;
}
impl From<FFIString> for String {
    fn from(value: FFIString) -> Self {
        let string: Result<Vec<u8>, _> = value.0.try_into();
        unsafe { String::from_utf8_unchecked(string.unwrap()) }
    }
}
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub enum OptionalFFIString {
    Some(FFIString),
    None,
}
impl FFIType for OptionalFFIString {
    type SafeType = Option<String>;
}
impl From<OptionalFFIString> for Option<String> {
    fn from(value: OptionalFFIString) -> Self {
        match value {
            OptionalFFIString::Some(string) => Some(string.into()),
            OptionalFFIString::None => None,
        }
    }
}
impl From<Option<String>> for OptionalFFIString {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(value) => Self::Some(value.into()),
            None => Self::None,
        }
    }
}
impl Serialize for OptionalFFIString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            OptionalFFIString::Some(string) => string.serialize(serializer),
            OptionalFFIString::None => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for OptionalFFIString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<String>::deserialize(deserializer)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}
impl Default for OptionalFFIString {
    fn default() -> Self {
        Self::None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn ffi_string() {
        let string = "Hello World".to_string();
        let ffi_string = FFIString::try_from(string.clone()).unwrap();
        let string2: String = ffi_string.try_into().unwrap();
        assert_eq!(string, string2);
    }
    #[test]
    fn ffi_string_default() {
        let ffi_string = FFIString::default();
        let string: String = ffi_string.try_into().unwrap();
        assert_eq!(string, "");
    }
    #[test]
    fn ffi_string_clone() {
        let ffi_string = FFIString::try_from("Hello World".to_string()).unwrap();
        let ffi_string2 = ffi_string.clone();
        assert_eq!(ffi_string, ffi_string2);
    }
    #[test]
    fn ffi_string_display() {
        let ffi_string = FFIString::try_from("Hello World".to_string()).unwrap();
        let string = format!("{}", ffi_string);
        assert_eq!(string, "Hello World");
    }
    #[test]
    fn optional_ffi_string() {
        let ffi_string = OptionalFFIString::try_from(Some("Hello World".to_string())).unwrap();
        let string: Option<String> = ffi_string.try_into().unwrap();
        assert_eq!(string.unwrap(), "Hello World");
    }
}
