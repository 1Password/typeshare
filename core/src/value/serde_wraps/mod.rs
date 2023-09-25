pub mod de;
pub mod se;

use super::Value;
use crate::value::{FromValueError, ToFromValue, ToValueError};
use serde::de::{DeserializeOwned, Unexpected};
use serde::Serialize;

use crate::value::serde_wraps::se::ValueSerializer;

impl Value {
    pub(crate) fn unexpected(&self) -> Unexpected {
        Unexpected::Other(self.into())
    }
}

impl<D: DeserializeOwned + Serialize> ToFromValue for D {
    fn from_value(value: Value) -> Result<Self, FromValueError>
    where
        Self: Sized,
    {
        D::deserialize(value)
    }

    fn to_value(&self) -> Result<Value, ToValueError>
    where
        Self: Sized,
    {
        self.serialize(ValueSerializer)
    }
}
#[cfg(test)]
mod tests {
    use crate::value::serde_wraps::ToFromValue;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
    pub struct TestStruct {
        pub a: u32,
        pub b: String,
    }
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub enum Enum {
        A,
        B(TestStruct),
        C { a: u32, b: String },
    }
    #[test]
    fn test_to_value() {
        let test_struct = TestStruct {
            a: 1,
            b: "test".to_string(),
        };
        let value = test_struct.to_value().unwrap();
        println!("{:?}", value);
        let test_struct1 = TestStruct::from_value(value).unwrap();
        assert_eq!(test_struct, test_struct1);
    }
    #[test]
    pub fn test_enum() {
        let vec = vec![
            Enum::A,
            Enum::B(TestStruct::default()),
            Enum::C {
                a: 1,
                b: "test".to_string(),
            },
        ];
        let value = vec.to_value().unwrap();
        println!("{:#?}", value);
        let vec1 = Vec::<Enum>::from_value(value).unwrap();
        assert_eq!(vec, vec1);
    }
}
