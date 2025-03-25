//! Length prefix encoding.

const THRESHOLDS: [(u64, u8, u8); 7] = [
    (0x100, 0x80, 1),             // 1 byte - 2^8, 0x80 = 2^8 - 2^7
    (0x10000, 0xc0, 2),           // 2 bytes - 2^16, 0xc0 = 2^8 - 2^6
    (0x1000000, 0xe0, 3),         // 3 bytes - 2^24, 0xe0 = 2^8 - 2^5
    (0x100000000, 0xf0, 4),       // 4 bytes - 2^32, 0xf0 = 2^8 - 2^4
    (0x10000000000, 0xf8, 5),     // 5 bytes - 2^40, 0xf8 = 2^8 - 2^3
    (0x1000000000000, 0xfc, 6),   // 6 bytes - 2^48, 0xfc = 2^8 - 2^2
    (0x100000000000000, 0xfe, 7), // 7 bytes - 2^56, 0xfe = 2^8 - 2^1
];

/// Encode a value into a length prefix.
fn encode(value: u64) -> Vec<u8> {
    // indicate the threshold and base
    let (threshold, base, length) = match value {
        0 => return vec![0],
        v if v < THRESHOLDS[0].0 => THRESHOLDS[0],
        v if v < THRESHOLDS[1].0 => THRESHOLDS[1],
        v if v < THRESHOLDS[2].0 => THRESHOLDS[2],
        v if v < THRESHOLDS[3].0 => THRESHOLDS[3],
        v if v < THRESHOLDS[4].0 => THRESHOLDS[4],
        v if v < THRESHOLDS[5].0 => THRESHOLDS[5],
        v if v < THRESHOLDS[6].0 => THRESHOLDS[6],
        _ => return vec![vec![255], value.to_le_bytes().to_vec()].concat(),
    };

    let mut encoded = vec![];

    // encode the length prefix
    encoded.push(base + (value / (base as u64)) as u8);
    encoded
}

#[ignore]
#[test]
fn thresholds() {
    for (i, threshold) in THRESHOLDS.iter().enumerate() {
        let length = i + 1;
        let value = threshold.0;
        assert_eq!(value.trailing_ones(), 8 * length as u32);

        // check that the base is correct
        let base = threshold.1;
        let expected = 2u64.pow(8 as u32) - 2u64.pow((8 - length) as u32);
        assert_eq!(base, expected as u8);
    }
}
