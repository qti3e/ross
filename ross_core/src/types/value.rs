use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

// TODO(qti3e) This file really needs to be improved, currently it just works
// but it doesn't work as intended.
// PrimitiveValue needs to be serialized untagged when using JSON, but be tagged
// with bincode.

/// Any primitive value that is supported by `core` to be put into an object tuple.
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
// #[serde(untagged)]
pub enum PrimitiveValue {
    Null,
    True,
    False,
    Hash16(Hash16),
    String(String),
    Number(Number),
}

/// A floating point number.
#[derive(Clone, PartialEq, PartialOrd, Eq, Serialize, Deserialize)]
pub struct Number(Option<NonNaN>);
#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
struct NonNaN(f64);
impl Eq for NonNaN {}

impl Number {
    /// Return true if the number is a NaN.
    pub fn is_nan(&self) -> bool {
        match self.0 {
            Some(_) => false,
            None => true,
        }
    }
}

impl Into<f64> for Number {
    fn into(self) -> f64 {
        match self.0 {
            Some(n) => n.0,
            _ => f64::NAN,
        }
    }
}

impl From<f64> for Number {
    fn from(v: f64) -> Number {
        if v.is_nan() {
            Number(None)
        } else {
            Number(Some(NonNaN(v)))
        }
    }
}

impl fmt::Debug for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(n) => write!(f, "{}", n.0),
            None => write!(f, "NaN"),
        }
    }
}

impl From<()> for PrimitiveValue {
    fn from(_: ()) -> Self {
        PrimitiveValue::Null
    }
}

impl From<String> for PrimitiveValue {
    fn from(v: String) -> Self {
        PrimitiveValue::String(v)
    }
}

impl From<bool> for PrimitiveValue {
    fn from(v: bool) -> Self {
        if v {
            PrimitiveValue::True
        } else {
            PrimitiveValue::False
        }
    }
}

impl From<Hash16> for PrimitiveValue {
    fn from(v: Hash16) -> Self {
        PrimitiveValue::Hash16(v)
    }
}

impl From<f64> for PrimitiveValue {
    fn from(v: f64) -> Self {
        PrimitiveValue::Number(v.into())
    }
}

#[cfg(test)]
mod test_primitive {
    use super::{Hash16, PrimitiveValue};

    macro_rules! json {
        ($value:expr) => {{
            let s = serde_json::to_string(&$value).unwrap();
            println!("JSON {:?}", &s);
            s
        }};
    }

    macro_rules! bincode {
        ($value:expr) => {
            bincode::serialize(&$value).unwrap()
        };
    }

    macro_rules! json_test {
        ($value:expr, $serialized:expr) => {{
            let ser = json!($value);
            // assert_eq!(ser, $serialized);
            assert_eq!(
                serde_json::from_str::<PrimitiveValue>(&ser).unwrap(),
                $value
            );
        }};
    }

    macro_rules! bincode_test {
        ($value:expr) => {
            assert_eq!(
                bincode::deserialize::<PrimitiveValue>(&bincode!($value)).unwrap(),
                $value
            );
        };
    }

    macro_rules! same {
        ($value:expr, $serialized:expr) => {{
            json_test!($value, $serialized);
            bincode_test!($value);
        }};
    }

    #[test]
    fn json() {
        println!("JSON");
        same!(PrimitiveValue::Null, "null");
        same!(PrimitiveValue::True, "true");
        same!(PrimitiveValue::False, "true");
        same!(PrimitiveValue::Number(6.0.into()), "6");
        same!(PrimitiveValue::Number(0.0.into()), "0");
        same!(
            PrimitiveValue::Hash16(Hash16::MIN),
            "\"00000000000000000000000000000000\""
        );
        same!(
            PrimitiveValue::Hash16(Hash16::MAX),
            "\"ffffffffffffffffffffffffffffffff\""
        );
        same!(
            PrimitiveValue::Hash16(Hash16::MAX),
            "\"ffffffffffffffffffffffffffffffff\""
        );
        same!(PrimitiveValue::String("Hello".into()), "\"Hello\"");
    }
}
