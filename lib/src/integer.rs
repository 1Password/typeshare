//! Integer types for use in the FFI layer.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};

const U53_MAX: u64 = 9_007_199_254_740_991;
#[allow(clippy::as_conversions)]
const I54_MAX: i64 = U53_MAX as i64;
const I54_MIN: i64 = -9_007_199_254_740_991;

/// Just like [`std::num::TryFromIntError`].
///
/// `std::num::TryFromIntError` cannot be constructed from outside of libstd so we have to provide
/// our own equivalent.
#[derive(Debug, PartialEq, Eq)]
pub struct TryFromIntError(());

// Error must implement Display for `#[serde(try_from = "FromType")]` https://serde.rs/container-attrs.html#try_from
impl Display for TryFromIntError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        write!(fmt, "Integer type conversion fail")
    }
}

macro_rules! truncated_type {
    ($truncated: ident, $untruncated: ident, $untruncated_str: expr, $min: expr, $max: expr, $doc: expr) => {
        #[doc = $doc]
        #[derive(
            Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash,
        )]
        #[serde(try_from = $untruncated_str)]
        pub struct $truncated($untruncated);

        impl $truncated {
            /// The smallest value that can be represented by this integer type.
            pub const MIN: $truncated = $truncated($min);

            /// The largest value that can be represented by this integer type.
            pub const MAX: $truncated = $truncated($max);
        }

        impl fmt::Debug for $truncated {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt::Debug::fmt(&self.0, fmt)
            }
        }

        impl fmt::Display for $truncated {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt::Display::fmt(&self.0, fmt)
            }
        }

        impl TryFrom<$untruncated> for $truncated {
            type Error = TryFromIntError;

            #[allow(unused_comparisons)]
            fn try_from(value: $untruncated) -> Result<Self, Self::Error> {
                if !($min..=$max).contains(&value) {
                    return Err(TryFromIntError(()));
                }

                Ok($truncated(value))
            }
        }

        impl From<$truncated> for $untruncated {
            fn from(value: $truncated) -> $untruncated {
                value.0
            }
        }

        impl PartialEq<$untruncated> for $truncated {
            fn eq(&self, other: &$untruncated) -> bool {
                self.0 == *other
            }
        }

        impl PartialOrd<$untruncated> for $truncated {
            fn partial_cmp(&self, other: &$untruncated) -> Option<Ordering> {
                Some(self.0.cmp(other))
            }
        }
    };
}

macro_rules! impl_truncated_type_from {
    ($from: ident, $into: ident) => {
        impl From<$into> for $from {
            fn from(value: $into) -> $from {
                $from(value.into())
            }
        }

        impl TryFrom<$from> for $into {
            type Error = TryFromIntError;

            #[allow(unused_comparisons)]
            fn try_from(value: $from) -> Result<$into, Self::Error> {
                if value.0 < $into::MIN.into() || value.0 > $into::MAX.into() {
                    return Err(TryFromIntError(()));
                }

                #[allow(clippy::as_conversions)]
                Ok(value.0 as $into)
            }
        }
    };
}

truncated_type!(
    U53,
    u64,
    "u64",
    0,
    U53_MAX,
    "The 53-bit unsigned integer type. Purpose of this type is to mimic JavaScript's integer type."
);
impl_truncated_type_from!(U53, u32);
impl_truncated_type_from!(U53, u16);
impl_truncated_type_from!(U53, u8);

truncated_type!(
    I54,
    i64,
    "i64",
    I54_MIN,
    I54_MAX,
    "The 54-bit signed integer type. Purpose of this type is to mimic JavaScript's integer type."
);
impl_truncated_type_from!(I54, i32);
impl_truncated_type_from!(I54, i16);
impl_truncated_type_from!(I54, i8);

/// Safely convert a `U53` integer to `usize`
#[inline]
#[must_use]
pub fn usize_from_u53_saturated(value: U53) -> usize {
    usize_from_u64_saturated(value.0)
}

/// Safely convert an unsigned 64-bit integer to `usize`
#[allow(clippy::as_conversions)]
#[inline]
#[must_use]
pub fn usize_from_u64_saturated(value: u64) -> usize {
    std::cmp::min(value, usize::MAX as u64) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    // I54
    #[test]
    fn test_i54_init() {
        assert_eq!(I54::try_from(I54_MAX).unwrap(), I54_MAX);
    }

    #[test]
    fn test_i54_overflow() {
        assert_eq!(I54::try_from(I54_MAX + 1), Err(TryFromIntError(())));
    }

    #[test]
    fn test_i54_underflow() {
        assert_eq!(I54::try_from(I54_MIN - 1), Err(TryFromIntError(())));
    }

    #[test]
    fn test_i64_to_i54() {
        assert_eq!(I54::try_from(i64::MAX), Err(TryFromIntError(())));
    }

    #[test]
    fn test_i54_to_i64() {
        assert_eq!(i64::from(I54::try_from(I54_MAX).unwrap()), I54_MAX);
    }

    #[test]
    fn test_i54_to_i32() {
        assert_eq!(i32::try_from(I54::from(i32::MAX)).unwrap(), i32::MAX);
    }

    #[test]
    #[allow(clippy::as_conversions)]
    fn test_i32_to_i54() {
        assert_eq!(I54::from(i32::MAX), i64::from(i32::MAX));
    }

    // U53
    #[test]
    fn test_u53_init() {
        assert_eq!(U53::try_from(U53_MAX).unwrap(), U53_MAX);
    }

    #[test]
    fn test_u53_overflow() {
        assert_eq!(U53::try_from(U53_MAX + 1), Err(TryFromIntError(())));
    }

    #[test]
    fn test_u64_to_u53() {
        assert_eq!(U53::try_from(u64::MAX), Err(TryFromIntError(())));
    }

    #[test]
    fn test_u53_to_u64() {
        assert_eq!(u64::from(U53::try_from(U53_MAX).unwrap()), U53_MAX);
    }

    #[test]
    fn test_u53_to_u32() {
        assert_eq!(u32::try_from(U53::from(u32::MAX)).unwrap(), u32::MAX);
    }

    #[test]
    #[allow(clippy::as_conversions)]
    fn test_u32_to_u53() {
        assert_eq!(U53::from(u32::MAX), u64::from(u32::MAX));
    }

    #[test]
    fn test_order() {
        assert!(U53::from(u32::MAX) < u64::MAX);
    }

    #[test]
    fn test_serde_serialize() {
        #[derive(Serialize)]
        struct Person {
            age: I54,
        }

        let j = serde_json::to_string(&Person { age: I54::from(12) }).unwrap();
        assert_eq!(j, r##"{"age":12}"##);
    }

    #[test]
    fn test_serde_deserialize() {
        #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
        struct Person {
            age: I54,
        }

        let j = r##"{"age":14}"##;
        assert_eq!(
            serde_json::from_str::<Person>(j).unwrap(),
            Person { age: I54::from(14) }
        );
    }

    #[test]
    fn test_serde_deserialize_overflow() {
        #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
        struct Person {
            age: I54,
        }

        let j = format!(r##"{{"age":{}}}"##, I54_MAX + 1);
        assert!(serde_json::from_str::<Person>(j.as_str()).is_err());
    }

    #[test]
    fn test_serde_deserialize_underflow() {
        #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
        struct Person {
            age: I54,
        }

        let j = format!(r##"{{"age":{}}}"##, I54_MIN - 1);
        assert!(serde_json::from_str::<Person>(j.as_str()).is_err());
    }

    #[test]
    fn test_formatter_flags() {
        let value: I54 = 125.into();

        // Right-aligned, include the + sign, width=8,
        assert_eq!(format!("{value:>+8}"), "    +125");
    }

    #[test]
    fn i54_max() {
        assert_eq!(I54_MAX, i64::from(I54::MAX));
    }

    #[test]
    fn i54_min() {
        assert_eq!(I54_MIN, i64::from(I54::MIN));
    }

    #[test]
    fn u53_max() {
        assert_eq!(U53_MAX, u64::from(U53::MAX));
    }

    #[test]
    fn u53_min() {
        assert_eq!(0, u64::from(U53::MIN));
    }
}
