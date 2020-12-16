use crate::utils::hash::Hash16;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::Formatter;
use std::marker::PhantomData;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum PrimitiveValue {
    Null,
    True,
    False,
    // TODO(qti3e) Add U32.
    Number(f64),
    Hash16(Hash16),
    String(String),
}

impl Eq for PrimitiveValue {}

impl From<bool> for PrimitiveValue {
    #[inline]
    fn from(value: bool) -> Self {
        if value {
            PrimitiveValue::True
        } else {
            PrimitiveValue::False
        }
    }
}

impl From<f64> for PrimitiveValue {
    #[inline]
    fn from(value: f64) -> Self {
        if value.is_finite() {
            PrimitiveValue::Number(value)
        } else {
            PrimitiveValue::Null
        }
    }
}

impl From<String> for PrimitiveValue {
    #[inline]
    fn from(value: String) -> Self {
        PrimitiveValue::String(value)
    }
}

impl<T: Into<PrimitiveValue>> From<Option<T>> for PrimitiveValue {
    #[inline]
    fn from(value: Option<T>) -> Self {
        if let Some(value) = value {
            value.into()
        } else {
            PrimitiveValue::Null
        }
    }
}

impl Serialize for PrimitiveValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            match &self {
                PrimitiveValue::Null => serializer.serialize_none(),
                PrimitiveValue::True => serializer.serialize_bool(true),
                PrimitiveValue::False => serializer.serialize_bool(false),
                PrimitiveValue::Number(n) => serializer.serialize_f64(*n),
                PrimitiveValue::Hash16(h) => h.serialize(serializer),
                PrimitiveValue::String(s) => serializer.serialize_str(&s),
            }
        } else {
            match &self {
                PrimitiveValue::Null => {
                    serializer.serialize_unit_variant("PrimitiveValue", 0, "Null")
                }
                PrimitiveValue::True => {
                    serializer.serialize_unit_variant("PrimitiveValue", 1, "True")
                }
                PrimitiveValue::False => {
                    serializer.serialize_unit_variant("PrimitiveValue", 2, "False")
                }
                PrimitiveValue::Number(value) => {
                    serializer.serialize_newtype_variant("PrimitiveValue", 3, "Number", value)
                }
                PrimitiveValue::Hash16(value) => {
                    serializer.serialize_newtype_variant("PrimitiveValue", 4, "Hash16", value)
                }
                PrimitiveValue::String(value) => {
                    serializer.serialize_newtype_variant("PrimitiveValue", 5, "String", value)
                }
            }
        }
    }
}

impl<'de> Deserialize<'de> for PrimitiveValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let content =
                match <serde::private::de::Content as Deserialize>::deserialize(deserializer) {
                    Ok(v) => v,
                    Err(err) => {
                        return Err(err);
                    }
                };

            if let Ok(ok) = Result::map(
                <f64 as Deserialize>::deserialize(serde::private::de::ContentRefDeserializer::<
                    D::Error,
                >::new(&content)),
                PrimitiveValue::Number,
            ) {
                return Ok(ok);
            }

            if let Ok(ok) = Result::map(
                <Hash16 as Deserialize>::deserialize(serde::private::de::ContentRefDeserializer::<
                    D::Error,
                >::new(&content)),
                PrimitiveValue::Hash16,
            ) {
                return Ok(ok);
            }

            if let Ok(ok) = Result::map(
                <String as Deserialize>::deserialize(serde::private::de::ContentRefDeserializer::<
                    D::Error,
                >::new(&content)),
                PrimitiveValue::String,
            ) {
                return Ok(ok);
            }

            if let Ok(ok) =
                <bool as Deserialize>::deserialize(serde::private::de::ContentRefDeserializer::<
                    D::Error,
                >::new(&content))
            {
                return Ok(ok.into());
            }

            if let Ok(_) =
                <() as Deserialize>::deserialize(serde::private::de::ContentRefDeserializer::<
                    D::Error,
                >::new(&content))
            {
                return Ok(PrimitiveValue::Null);
            }

            Err(de::Error::custom(
                "data did not match any variant of PrimitiveValue",
            ))
        } else {
            #[allow(non_camel_case_types)]
            enum Field {
                field0,
                field1,
                field2,
                field3,
                field4,
                field5,
            }

            struct FieldVisitor;
            impl<'de> serde::de::Visitor<'de> for FieldVisitor {
                type Value = Field;
                fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                    Formatter::write_str(formatter, "variant identifier")
                }
                fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    match value {
                        0u64 => Ok(Field::field0),
                        1u64 => Ok(Field::field1),
                        2u64 => Ok(Field::field2),
                        3u64 => Ok(Field::field3),
                        4u64 => Ok(Field::field4),
                        5u64 => Ok(Field::field5),
                        _ => Err(serde::de::Error::invalid_value(
                            serde::de::Unexpected::Unsigned(value),
                            &"variant index 0 <= i < 6",
                        )),
                    }
                }
            }
            impl<'de> serde::Deserialize<'de> for Field {
                #[inline]
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    serde::Deserializer::deserialize_identifier(deserializer, FieldVisitor)
                }
            }
            struct Visitor<'de> {
                marker: PhantomData<PrimitiveValue>,
                lifetime: PhantomData<&'de ()>,
            }
            impl<'de> serde::de::Visitor<'de> for Visitor<'de> {
                type Value = PrimitiveValue;
                fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                    Formatter::write_str(formatter, "enum PrimitiveValue")
                }
                fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::EnumAccess<'de>,
                {
                    match match serde::de::EnumAccess::variant(data) {
                        Ok(val) => val,
                        Err(err) => {
                            return Err(err);
                        }
                    } {
                        (Field::field0, variant) => {
                            match serde::de::VariantAccess::unit_variant(variant) {
                                Ok(val) => val,
                                Err(err) => {
                                    return Err(err);
                                }
                            };
                            Ok(PrimitiveValue::Null)
                        }
                        (Field::field1, variant) => {
                            match serde::de::VariantAccess::unit_variant(variant) {
                                Ok(val) => val,
                                Err(err) => {
                                    return Err(err);
                                }
                            };
                            Ok(PrimitiveValue::True)
                        }
                        (Field::field2, variant) => {
                            match serde::de::VariantAccess::unit_variant(variant) {
                                Ok(val) => val,
                                Err(err) => {
                                    return Err(err);
                                }
                            };
                            Ok(PrimitiveValue::False)
                        }
                        (Field::field3, variant) => Result::map(
                            serde::de::VariantAccess::newtype_variant::<f64>(variant),
                            PrimitiveValue::Number,
                        ),
                        (Field::field4, variant) => Result::map(
                            serde::de::VariantAccess::newtype_variant::<Hash16>(variant),
                            PrimitiveValue::Hash16,
                        ),
                        (Field::field5, variant) => Result::map(
                            serde::de::VariantAccess::newtype_variant::<String>(variant),
                            PrimitiveValue::String,
                        ),
                    }
                }
            }
            const VARIANTS: &'static [&'static str] =
                &["Null", "True", "False", "Number", "Hash16", "String"];
            serde::Deserializer::deserialize_enum(
                deserializer,
                "PrimitiveValue",
                VARIANTS,
                Visitor {
                    marker: PhantomData::<PrimitiveValue>,
                    lifetime: PhantomData,
                },
            )
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Hash16, PrimitiveValue};
    use bincode::Options;

    macro_rules! json {
        ($value:expr) => {{
            let s = serde_json::to_string(&$value).unwrap();
            s
        }};
    }

    macro_rules! bincode {
        ($value:expr) => {{
            let x = bincode::DefaultOptions::default()
                .with_varint_encoding()
                .serialize(&$value).unwrap();
            println!("{:?} -> {:?} ({} bytes)", $value, x, x.len());
            x
        }};
    }

    macro_rules! json_test {
        ($value:expr, $serialized:expr) => {{
            let ser = json!($value);
            assert_eq!(ser, $serialized);
            assert_eq!(
                serde_json::from_str::<PrimitiveValue>(&ser).unwrap(),
                $value
            );
        }};
    }

    macro_rules! bincode_test {
        ($value:expr) => {
            assert_eq!(
                bincode::DefaultOptions::default()
                    .with_varint_encoding()
                    .deserialize::<PrimitiveValue>(&bincode!($value)).unwrap(),
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
        same!(PrimitiveValue::Null, "null");
        same!(PrimitiveValue::True, "true");
        same!(PrimitiveValue::False, "false");
        same!(PrimitiveValue::Number(6.0.into()), "6.0");
        same!(PrimitiveValue::Number(0.0.into()), "0.0");
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
