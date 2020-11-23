use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{self, Write};

/// A hash that occupies exactly 16 bytes, this is meant to be serialized and
/// deserialized using bincode, to use JSON one must first convert this to an
/// string and then perform the serialization.
///
/// 16 bytes can be used to store UUIDs.
#[derive(Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize, Hash, Copy, Clone)]
pub struct Hash16([u8; 16]);

impl Hash16 {
    pub const MIN: Hash16 = Hash16([0; 16]);
    pub const MAX: Hash16 = Hash16([255; 16]);

    pub fn parse(s: &str) -> Result<Hash16, HashParseError> {
        if s.len() != 32 {
            return Err(HashParseError::InsufficientLength);
        }
        Ok(Hash16([
            u8::from_str_radix(&s[0..2], 16).map_err(|_| HashParseError::RadixError(0))?,
            u8::from_str_radix(&s[2..4], 16).map_err(|_| HashParseError::RadixError(2))?,
            u8::from_str_radix(&s[4..6], 16).map_err(|_| HashParseError::RadixError(4))?,
            u8::from_str_radix(&s[6..8], 16).map_err(|_| HashParseError::RadixError(6))?,
            u8::from_str_radix(&s[8..10], 16).map_err(|_| HashParseError::RadixError(8))?,
            u8::from_str_radix(&s[10..12], 16).map_err(|_| HashParseError::RadixError(10))?,
            u8::from_str_radix(&s[12..14], 16).map_err(|_| HashParseError::RadixError(12))?,
            u8::from_str_radix(&s[14..16], 16).map_err(|_| HashParseError::RadixError(14))?,
            u8::from_str_radix(&s[16..18], 16).map_err(|_| HashParseError::RadixError(16))?,
            u8::from_str_radix(&s[18..20], 16).map_err(|_| HashParseError::RadixError(18))?,
            u8::from_str_radix(&s[20..22], 16).map_err(|_| HashParseError::RadixError(20))?,
            u8::from_str_radix(&s[22..24], 16).map_err(|_| HashParseError::RadixError(22))?,
            u8::from_str_radix(&s[24..26], 16).map_err(|_| HashParseError::RadixError(24))?,
            u8::from_str_radix(&s[26..28], 16).map_err(|_| HashParseError::RadixError(26))?,
            u8::from_str_radix(&s[28..30], 16).map_err(|_| HashParseError::RadixError(28))?,
            u8::from_str_radix(&s[30..32], 16).map_err(|_| HashParseError::RadixError(30))?,
        ]))
    }
}

impl From<[u8; 16]> for Hash16 {
    fn from(slice: [u8; 16]) -> Self {
        Hash16(slice)
    }
}

impl From<&Hash16> for String {
    fn from(uuid: &Hash16) -> Self {
        static CHARS: &'static [u8] = b"0123456789abcdef";
        let mut s = String::with_capacity(32);

        for &byte in uuid.0.iter() {
            s.write_char(CHARS[(byte >> 4) as usize].into()).unwrap();
            s.write_char(CHARS[(byte & 0xf) as usize].into()).unwrap()
        }

        s
    }
}

impl std::fmt::Debug for Hash16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&String::from(self))
    }
}

/// A hash that occupies exactly 20 bytes, this is meant to be serialized and
/// deserialized using bincode, to use JSON one must first convert this to an
/// string and then perform the serialization.
///
/// 16 bytes can be used to store SHA-1.
#[derive(Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize, Hash, Copy, Clone)]
pub struct Hash20([u8; 20]);

impl Hash20 {
    pub const MIN: Hash20 = Hash20([0; 20]);
    pub const MAX: Hash20 = Hash20([255; 20]);

    pub fn parse(s: &str) -> Result<Hash20, HashParseError> {
        if s.len() != 40 {
            return Err(HashParseError::InsufficientLength);
        }
        Ok(Hash20([
            u8::from_str_radix(&s[0..2], 16).map_err(|_| HashParseError::RadixError(0))?,
            u8::from_str_radix(&s[2..4], 16).map_err(|_| HashParseError::RadixError(2))?,
            u8::from_str_radix(&s[4..6], 16).map_err(|_| HashParseError::RadixError(4))?,
            u8::from_str_radix(&s[6..8], 16).map_err(|_| HashParseError::RadixError(6))?,
            u8::from_str_radix(&s[8..10], 16).map_err(|_| HashParseError::RadixError(8))?,
            u8::from_str_radix(&s[10..12], 16).map_err(|_| HashParseError::RadixError(10))?,
            u8::from_str_radix(&s[12..14], 16).map_err(|_| HashParseError::RadixError(12))?,
            u8::from_str_radix(&s[14..16], 16).map_err(|_| HashParseError::RadixError(14))?,
            u8::from_str_radix(&s[16..18], 16).map_err(|_| HashParseError::RadixError(16))?,
            u8::from_str_radix(&s[18..20], 16).map_err(|_| HashParseError::RadixError(18))?,
            u8::from_str_radix(&s[20..22], 16).map_err(|_| HashParseError::RadixError(20))?,
            u8::from_str_radix(&s[22..24], 16).map_err(|_| HashParseError::RadixError(22))?,
            u8::from_str_radix(&s[24..26], 16).map_err(|_| HashParseError::RadixError(24))?,
            u8::from_str_radix(&s[26..28], 16).map_err(|_| HashParseError::RadixError(26))?,
            u8::from_str_radix(&s[28..30], 16).map_err(|_| HashParseError::RadixError(28))?,
            u8::from_str_radix(&s[30..32], 16).map_err(|_| HashParseError::RadixError(30))?,
            u8::from_str_radix(&s[32..34], 16).map_err(|_| HashParseError::RadixError(32))?,
            u8::from_str_radix(&s[34..36], 16).map_err(|_| HashParseError::RadixError(34))?,
            u8::from_str_radix(&s[36..38], 16).map_err(|_| HashParseError::RadixError(36))?,
            u8::from_str_radix(&s[38..40], 16).map_err(|_| HashParseError::RadixError(38))?,
        ]))
    }
}

impl From<[u8; 20]> for Hash20 {
    fn from(slice: [u8; 20]) -> Self {
        Hash20(slice)
    }
}

impl From<&Hash20> for String {
    fn from(uuid: &Hash20) -> Self {
        static CHARS: &'static [u8] = b"0123456789abcdef";
        let mut s = String::with_capacity(32);

        for &byte in uuid.0.iter() {
            s.write_char(CHARS[(byte >> 4) as usize].into()).unwrap();
            s.write_char(CHARS[(byte & 0xf) as usize].into()).unwrap()
        }

        s
    }
}

impl std::fmt::Debug for Hash20 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&String::from(self))
    }
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

#[cfg(test)]
mod test_hash16 {
    use super::*;

    #[test]
    fn serde_bincode() {
        let uuid = Hash16::parse("5e78dc74efe74338a7a4d6d16d655e52").unwrap();
        let serialized = bincode::serialize(&uuid).unwrap();
        assert_eq!(serialized.len(), 16);
        let deserialized = bincode::deserialize::<Hash16>(&serialized).unwrap();
        assert_eq!(uuid, deserialized);
    }

    #[test]
    fn to_string() {
        let s = "5e78dc74efe74338a7a4d6d16d655e52";
        let uuid = Hash16::parse(s).unwrap();
        assert_eq!(String::from(&uuid), s);
    }

    #[test]
    fn min_max() {
        let min = "00000000000000000000000000000000";
        let max = "ffffffffffffffffffffffffffffffff";
        assert_eq!(String::from(&Hash16::MIN), min);
        assert_eq!(String::from(&Hash16::MAX), max);
    }
}

#[cfg(test)]
mod test_hash20 {
    use super::*;

    #[test]
    fn serde_bincode() {
        let uuid = Hash20::parse("f45fb68d900054e8cd89b120957bd3dcb2d8dede").unwrap();
        let serialized = bincode::serialize(&uuid).unwrap();
        assert_eq!(serialized.len(), 20);
        let deserialized = bincode::deserialize::<Hash20>(&serialized).unwrap();
        assert_eq!(uuid, deserialized);
    }

    #[test]
    fn to_string() {
        let s = "f45fb68d900054e8cd89b120957bd3dcb2d8dede";
        let uuid = Hash20::parse(s).unwrap();
        assert_eq!(String::from(&uuid), s);
    }

    #[test]
    fn min_max() {
        let min = "0000000000000000000000000000000000000000";
        let max = "ffffffffffffffffffffffffffffffffffffffff";
        assert_eq!(String::from(&Hash20::MIN), min);
        assert_eq!(String::from(&Hash20::MAX), max);
    }
}
