//! Length prefix encoding.

use crate::compact::Numeric;

/// The thresholds for the length prefix encoding.
const THRESHOLDS: [(usize, u8, u8, u64, u64); 7] = [
    // 1 bytes, 0x80 = 2^8 - 2^7, 2^8, 2^14
    (1, 0x80, 0xc0, 0x100, 0x4000),
    // 2 bytes, 0xc0 = 2^8 - 2^6, 2^16, 2^21
    (2, 0xc0, 0xe0, 0x10000, 0x200000),
    // 3 bytes, 0xe0 = 2^8 - 2^5, 2^24, 2^28
    (3, 0xe0, 0xf0, 0x1000000, 0x10000000),
    // 4 bytes, 0xf0 = 2^8 - 2^4, 2^32, 2^35
    (4, 0xf0, 0xf8, 0x100000000, 0x800000000),
    // 5 bytes, 0xf8 = 2^8 - 2^3, 2^40, 2^42
    (5, 0xf8, 0xfc, 0x10000000000, 0x40000000000),
    // 6 bytes, 0xfc = 2^8 - 2^2, 2^48, 2^49
    (6, 0xfc, 0xfe, 0x1000000000000, 0x2000000000000),
    // 7 bytes, 0xfe = 2^8 - 2^1, 2^56, 2^56
    (7, 0xfe, 0xff, 0x100000000000000, 0x100000000000000),
];

/// Encode a value into a length prefix.
pub fn encode(value: u64) -> Vec<u8> {
    if value < 0x80 {
        return vec![value as u8];
    }

    // It's okay to use for loops here because it has the same performance as a match.
    for (_, base, _, bits, threshold) in THRESHOLDS.into_iter() {
        if value < threshold {
            let mut encoded = vec![base + (value / bits) as u8];
            let remainder = (value % bits).encode();
            if remainder.is_empty() {
                encoded.push(0);
            } else {
                encoded.extend_from_slice(&remainder);
            }

            return encoded;
        }
    }

    [vec![255], value.to_le_bytes().to_vec()].concat()
}

/// Decode a compact encoded number.
pub fn decode(encoded: &[u8]) -> u64 {
    self::decode_from(encoded).0
}

/// Decode a length prefix.
pub fn decode_from(encoded: &[u8]) -> (u64, usize) {
    if encoded.is_empty() {
        return (0, 0);
    }

    let prefix = encoded[0];
    if prefix < 0x80 {
        return (prefix as u64, 1);
    }

    // loop instead match for returning the length
    for (length, base, next, bits, _) in THRESHOLDS.into_iter() {
        if prefix < next {
            let dlen = length + 1;
            return (
                ((prefix - base) as u64) * bits + u64::decode(&encoded[1..dlen]),
                dlen,
            );
        }
    }

    (u64::decode(&encoded[1..9]), 9)
}

#[test]
fn thresholds() {
    for (length, base, _, bits, threshold) in THRESHOLDS.into_iter() {
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
    for (length, _, _, _, threshold) in THRESHOLDS.iter() {
        let value = threshold - 1;
        let encoded = encode(value);
        let (decoded, dlen) = decode_from(&encoded);
        assert_eq!(dlen, *length + 1);
        assert_eq!(encoded.len(), dlen);
        assert_eq!(value, decoded);
    }
}
