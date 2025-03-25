//! Instruction format with one register and one extended immediate.

use crate::format::REI;

impl From<REI> for Vec<u8> {
    fn from(value: REI) -> Self {
        let mut result = Vec::with_capacity(9);
        result.push(value.reg0 & 0xF);

        // Add extended immediate (8-byte value in little-endian format)
        result.extend_from_slice(&value.eimm0.to_le_bytes());
        result
    }
}

impl From<&[u8]> for REI {
    fn from(bytes: &[u8]) -> Self {
        // Must have at least 9 bytes: register (1) + extended immediate (8)
        if bytes.len() < 9 {
            // mb change the decoding functions to `TryFrom`
            todo!("Not enough bytes to decode REI format");
        }

        // Extract register from the lower 4 bits of the first byte
        let reg0 = bytes[0] & 0xF;

        // Extract extended immediate from the next 8 bytes (little-endian)
        let mut immediate_bytes = [0u8; 8];
        immediate_bytes.copy_from_slice(&bytes[1..9]);
        let eimm0 = u64::from_le_bytes(immediate_bytes);

        REI { reg0, eimm0 }
    }
}

#[test]
fn test_rei_encoding() {
    let eimm0 = 0x0000000000ABCDEF;
    let reg0 = 7;
    let original = REI { reg0, eimm0 };

    // Encode the REI
    let encoded: Vec<u8> = original.into();

    // Check encoding format
    assert_eq!(encoded.len(), 9); // 1 byte for register + 8 bytes for extended immediate
    assert_eq!(encoded[0], reg0); // Register should be 7

    // Decode back to REI
    let decoded = REI::from(encoded.as_ref());

    // Check that decoded matches original values
    assert_eq!(decoded.reg0, reg0);
    assert_eq!(decoded.eimm0, eimm0);
}
