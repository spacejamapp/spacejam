//! Length prefix encoding.

use crate::{Numeric, Result};

/// The thresholds for the length prefix encoding.
const THRESHOLDS: [(u8, u8, u64, u64); 7] = [
    // 1 bytes, 0x80 = 2^8 - 2^7, 2^8, 2^14
    (1, 0x80, 0x100, 0x4000),
    // 2 bytes, 0xc0 = 2^8 - 2^6, 2^16, 2^21
    (2, 0xc0, 0x10000, 0x200000),
    // 3 bytes, 0xe0 = 2^8 - 2^5, 2^24, 2^28
    (3, 0xe0, 0x1000000, 0x10000000),
    // 4 bytes, 0xf0 = 2^8 - 2^4, 2^32, 2^35
    (4, 0xf0, 0x100000000, 0x800000000),
    // 5 bytes, 0xf8 = 2^8 - 2^3, 2^40, 2^42
    (5, 0xf8, 0x10000000000, 0x40000000000),
    // 6 bytes, 0xfc = 2^8 - 2^2, 2^48, 2^49
    (6, 0xfc, 0x1000000000000, 0x2000000000000),
    // 7 bytes, 0xfe = 2^8 - 2^1, 2^56, 2^56
    (7, 0xfe, 0x100000000000000, 0x100000000000000),
];

/// Encode a value into a length prefix.
pub fn encode(value: u64) -> Vec<u8> {
    // indicate the threshold and base
    let (_, base, bits, _) = match value {
        v if v < 0x80 => return vec![v as u8],
        v if v < THRESHOLDS[0].3 => THRESHOLDS[0],
        v if v < THRESHOLDS[1].3 => THRESHOLDS[1],
        v if v < THRESHOLDS[2].3 => THRESHOLDS[2],
        v if v < THRESHOLDS[3].3 => THRESHOLDS[3],
        v if v < THRESHOLDS[4].3 => THRESHOLDS[4],
        v if v < THRESHOLDS[5].3 => THRESHOLDS[5],
        v if v < THRESHOLDS[6].3 => THRESHOLDS[6],
        _ => return vec![vec![255], value.to_le_bytes().to_vec()].concat(),
    };

    // encode the length prefix and the resut
    let mut encoded = vec![];
    encoded.push(base + (value / bits) as u8);
    encoded.extend_from_slice(&(value % bits).encode());
    encoded
}

/// Decode a length prefix.
pub fn decode(encoded: &[u8]) -> Result<u64> {
    match encoded[0] {
        v if v < 0x80 => Ok(v as u64),
        v if v < 0xc0 => Ok((v as u64 - 0x80) * 0x100 + u64::decode(&encoded[1..])),
        v if v < 0xe0 => Ok((v as u64 - 0xc0) * 0x10000 + u64::decode(&encoded[1..])),
        v if v < 0xf0 => Ok((v as u64 - 0xe0) * 0x1000000 + u64::decode(&encoded[1..])),
        v if v < 0xf8 => Ok((v as u64 - 0xf0) * 0x100000000 + u64::decode(&encoded[1..])),
        v if v < 0xfc => Ok((v as u64 - 0xf8) * 0x10000000000 + u64::decode(&encoded[1..])),
        v if v < 0xfe => Ok((v as u64 - 0xfc) * 0x1000000000000 + u64::decode(&encoded[1..])),
        _ => Ok(u64::decode(&encoded[1..])),
    }
}

#[test]
fn thresholds() {
    for (length, base, bits, threshold) in THRESHOLDS.into_iter() {
        assert_eq!(bits.trailing_zeros(), 8 * length as u32);

        // check that the base is correct
        let expected = 2u64.pow(8 as u32) - 2u64.pow((8 - length) as u32);
        assert_eq!(base, expected as u8);

        // check the threshold is correct
        let expected = 2u64.pow(7 * (length + 1) as u32);
        assert_eq!(threshold, expected,);
    }
}

#[test]
fn roundtrip() {
    for i in THRESHOLDS.iter().map(|(_, _, _, threshold)| threshold) {
        let value = i - 1;
        let encoded = encode(value);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(value, decoded);
    }
}
