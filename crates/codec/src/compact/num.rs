//! Number encoding and decoding

use crate::compact;
use serde::Serialize;

/// Trait for types that can be encoded and decoded using JAMCodec
pub trait Numeric: Sized + Default + Copy + Serialize {
    const LENGTH: usize;

    /// Encode the value into a byte vector
    fn encode(&self) -> Vec<u8>;

    /// Decode the value from a byte vector
    fn decode(value: &[u8]) -> Self;

    /// Encode the value into a compact byte vector
    fn compact_encode(&self) -> Vec<u8>;

    /// Decode the value from a compact byte vector
    fn compact_decode(source: &[u8]) -> Self;

    /// Create a numeric value from a u64
    fn from_u64(value: u64) -> Self;
}

/// Implement the `Numeric` trait for the given types.
macro_rules! impl_numeric {
    ($(($t:ty, $len:expr)),+) => {
        $(
            impl Numeric for $t {
                const LENGTH: usize = $len;

                fn encode(&self) -> Vec<u8> {
                    let bytes = self.to_le_bytes().to_vec();
                    let end = $len - self.leading_zeros() as usize / 8;
                    bytes[..end].to_vec()
                }

                fn decode(source: &[u8]) -> Self {
                    let len = source.len();
                    let mut bytes = [0; $len];
                    bytes[0..len.min($len)].copy_from_slice(source);
                    Self::from_le_bytes(bytes)
                }

                fn compact_encode(&self) -> Vec<u8> {
                    compact::encode(*self as u64)
                }

                fn compact_decode(source: &[u8]) -> Self {
                    compact::decode(source) as $t
                }

                fn from_u64(value: u64) -> Self {
                    value as $t
                }
            }
        )+
    }
}

impl_numeric! {
    (i8, 1),
    (u8, 1),
    (i16, 2),
    (u16, 2),
    (i32, 4),
    (u32, 4),
    (i64, 8),
    (u64, 8)
}

#[cfg(test)]
macro_rules! test_codec {
    ($t:ty, $source:expr) => {
        let value = <$t>::from_le_bytes($source);
        let encoded = value.encode();
        let decoded = <$t>::decode(&encoded);
        assert_eq!(value, decoded);
        assert_eq!(
            encoded.len(),
            $source.len() - value.leading_zeros() as usize / 8,
        );
    };
}

#[test]
fn i8() {
    let values = vec![-128, -1, 0, 1, 127];
    for value in values {
        let encoded = value.encode();
        let decoded = i8::decode(&encoded);
        assert_eq!(value, decoded);
    }
}

#[test]
fn u8() {
    let values = vec![0, 1, 127, 128, 255];
    for value in values {
        let encoded = value.encode();
        let decoded = u8::decode(&encoded);
        assert_eq!(value, decoded);
    }
}

#[test]
fn i16() {
    let values = vec![[255, 0], [0, 255]];
    for source in values {
        test_codec!(i16, source);
    }
}

#[test]
fn u16() {
    let values = vec![[255, 0], [0, 255]];
    for source in values {
        test_codec!(u16, source);
    }
}

#[test]
fn i32() {
    let values = vec![
        [255, 0, 0, 0],
        [0, 255, 0, 0],
        [0, 0, 255, 0],
        [0, 0, 0, 255],
    ];
    for source in values {
        test_codec!(i32, source);
    }
}

#[test]
fn u32() {
    let values = vec![
        [255, 0, 0, 0],
        [0, 255, 0, 0],
        [0, 0, 255, 0],
        [0, 0, 0, 255],
    ];
    for source in values {
        test_codec!(u32, source);
    }
}

#[test]
fn i64() {
    let values = vec![
        [255, 0, 0, 0, 0, 0, 0, 0],
        [0, 255, 0, 0, 0, 0, 0, 0],
        [0, 0, 255, 0, 0, 0, 0, 0],
        [0, 0, 0, 255, 0, 0, 0, 0],
        [0, 0, 0, 0, 255, 0, 0, 0],
        [0, 0, 0, 0, 0, 255, 0, 0],
        [0, 0, 0, 0, 0, 0, 255, 0],
        [0, 0, 0, 0, 0, 0, 0, 255],
    ];
    for source in values {
        test_codec!(i64, source);
    }
}

#[test]
fn u64() {
    let values = vec![
        [255, 0, 0, 0, 0, 0, 0, 0],
        [0, 255, 0, 0, 0, 0, 0, 0],
        [0, 0, 255, 0, 0, 0, 0, 0],
        [0, 0, 0, 255, 0, 0, 0, 0],
        [0, 0, 0, 0, 255, 0, 0, 0],
        [0, 0, 0, 0, 0, 255, 0, 0],
        [0, 0, 0, 0, 0, 0, 255, 0],
        [0, 0, 0, 0, 0, 0, 0, 255],
    ];
    for source in values {
        test_codec!(u64, source);
    }
}
