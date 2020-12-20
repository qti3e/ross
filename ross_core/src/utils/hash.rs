use rand::distributions::{Distribution, Standard};
use rand::Rng;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::fmt::{self, Write};
use std::str::FromStr;

macro_rules! hash_n {
    ($(#[$attr:meta])* $name:ident($size:expr)) => {
        $(#[$attr])*
        #[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
        pub struct $name([u8; $size]);

        impl $name {
            pub const MIN: Self = Self([0; $size]);
            pub const MAX: Self = Self([255; $size]);
        }

        impl Distribution<$name> for Standard {
            fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> $name {
                $name(rng.gen())
            }
        }

        impl FromStr for $name {
            type Err = HashParseError;

            fn from_str(string: &str) -> Result<Self, Self::Err> {
                if string.len() != $size * 2 {
                    return Err(HashParseError::InsufficientLength);
                }

                let mut buf = [0; $size];
                for i in 0..$size {
                    let s = i * 2;
                    buf[i] = u8::from_str_radix(&string[s..s + 2], 16)
                        .map_err(|_| HashParseError::RadixError(s as u8))?;
                }

                Ok(Self(buf))
            }
        }

        impl TryFrom<&[u8]> for $name {
            type Error = HashParseError;

            fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
                if slice.len() != $size {
                    return Err(HashParseError::InsufficientLength);
                }
                let mut s = [0; $size];
                s.copy_from_slice(slice);
                Ok(Self(s))
            }
        }

        impl From<[u8; $size]> for $name {
            fn from(slice: [u8; $size]) -> Self {
                Self(slice)
            }
        }

        impl From<&$name> for String {
            fn from(uuid: &$name) -> Self {
                static CHARS: &'static [u8] = b"0123456789abcdef";
                let mut s = String::with_capacity($size * 2);

                for &byte in uuid.0.iter() {
                    s.write_char(CHARS[(byte >> 4) as usize].into()).unwrap();
                    s.write_char(CHARS[(byte & 0xf) as usize].into()).unwrap()
                }

                s
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&String::from(self))
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                if serializer.is_human_readable() {
                    serializer.serialize_str(&String::from(self))
                } else {
                    serializer.serialize_newtype_struct("Hash", &self.0)
                }
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                if deserializer.is_human_readable() {
                    struct HashStringVisitor;

                    impl<'vi> de::Visitor<'vi> for HashStringVisitor {
                        type Value = $name;

                        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                            write!(formatter, "a hash string")
                        }

                        #[inline]
                        fn visit_str<E: de::Error>(self, value: &str) -> Result<$name, E> {
                            value.parse::<$name>().map_err(E::custom)
                        }

                        #[inline]
                        fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<$name, E> {
                            value.try_into().map_err(E::custom)
                        }
                    }

                    deserializer.deserialize_str(HashStringVisitor)
                } else {
                    struct HashBytesVisitor;

                    impl<'vi> de::Visitor<'vi> for HashBytesVisitor {
                        type Value = $name;

                        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                            write!(formatter, "bytes")
                        }

                        #[inline]
                        fn visit_newtype_struct<V>(
                            self,
                            value: V,
                        ) -> serde::export::Result<Self::Value, V::Error>
                        where
                            V: Deserializer<'vi>,
                        {
                            let slice: [u8; $size] =
                                match <[u8; $size] as serde::Deserialize>::deserialize(value) {
                                    Ok(val) => val,
                                    Err(err) => {
                                        return Err(err);
                                    }
                                };
                            Ok($name(slice))
                        }

                        #[inline]
                        fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
                        where
                            S: de::SeqAccess<'vi>,
                        {
                            let slice =
                                match match de::SeqAccess::next_element::<[u8; $size]>(&mut seq) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        return Err(e);
                                    }
                                } {
                                    Some(value) => value,
                                    None => {
                                        return Err(de::Error::invalid_length(
                                            0usize,
                                            &"hash requires an slice.",
                                        ));
                                    }
                                };
                            Ok($name(slice))
                        }
                    }

                    deserializer.deserialize_newtype_struct("Hash", HashBytesVisitor)
                }
            }
        }
    };
}

#[derive(Debug)]
pub enum HashParseError {
    InsufficientLength,
    RadixError(u8),
}

impl fmt::Display for HashParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            HashParseError::InsufficientLength => write!(f, "Length of the hash was not correct."),
            HashParseError::RadixError(pos) => {
                write!(f, "Couldn't parse hex string at position {}", pos)
            }
        }
    }
}

impl Error for HashParseError {}

hash_n!(
    /// A 16-byte hash that occupies exactly 16 bytes when serialized using bincode,
    /// and can also be serialized/deserialized to/from an string when targeting JSON
    /// 16-byte is enough for standard UUIDs.
    Hash16(16)
);

hash_n!(
    /// Like `Hash16` but is exactly 20 bytes which is enough for storing SHA-1.
    Hash20(20)
);

#[cfg(test)]
mod test {
    use super::{Hash16, Hash20};

    #[test]
    fn bincode_16() {
        let s = "5e78dc74efe74338a7a4d6d16d655e52";
        let id = s.parse::<Hash16>().unwrap();
        let ser = bincode::serialize(&id).unwrap();
        assert_eq!(ser.len(), 16);
        let de = bincode::deserialize::<Hash16>(&ser).unwrap();
        assert_eq!(de, id);
    }

    #[test]
    fn json_16() {
        let s = "5e78dc74efe74338a7a4d6d16d655e52";
        let id = s.parse::<Hash16>().unwrap();
        let ser = serde_json::to_string(&id).unwrap();
        assert_eq!(format!("\"{}\"", s), ser);
        let de = serde_json::from_str::<Hash16>(&ser).unwrap();
        assert_eq!(de, id);
    }

    #[test]
    fn bincode_20() {
        let s = "f45fb68d900054e8cd89b120957bd3dcb2d8dede";
        let id = s.parse::<Hash20>().unwrap();
        let ser = bincode::serialize(&id).unwrap();
        assert_eq!(ser.len(), 20);
        let de = bincode::deserialize::<Hash20>(&ser).unwrap();
        assert_eq!(de, id);
    }

    #[test]
    fn json_20() {
        let s = "f45fb68d900054e8cd89b120957bd3dcb2d8dede";
        let id = s.parse::<Hash20>().unwrap();
        let ser = serde_json::to_string(&id).unwrap();
        assert_eq!(format!("\"{}\"", s), ser);
        let de = serde_json::from_str::<Hash20>(&ser).unwrap();
        assert_eq!(de, id);
    }

    #[test]
    fn random() {
        let x: Hash16 = rand::random();
        let y: Hash16 = rand::random();
        assert_eq!(x, x);
        assert_ne!(x, y);
        let x: Hash20 = rand::random();
        let y: Hash20 = rand::random();
        assert_eq!(x, x);
        assert_ne!(x, y);
    }
}
