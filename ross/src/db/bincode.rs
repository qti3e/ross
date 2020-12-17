//! This module contains serialize and deserialize functions with custom
//! configurations.

use bincode::Options;

#[inline(always)]
pub fn serialize<S: ?Sized + serde::Serialize>(t: &S) -> Vec<u8> {
    bincode::DefaultOptions::new()
        .with_varint_encoding()
        .serialize(t)
        .unwrap()
}

#[inline(always)]
pub fn deserialize<'a, T: serde::Deserialize<'a>>(bytes: &'a [u8]) -> T {
    bincode::DefaultOptions::new()
        .with_varint_encoding()
        .deserialize(bytes)
        .unwrap()
}
