//! Instructions with one register and one immediate.

use crate::format::{ISA, RI};

impl From<RI> for Vec<u8> {
    fn from(value: RI) -> Vec<u8> {
        let x_len = value.imm0.len();
        let mut bytes = Vec::with_capacity(1 + x_len as usize);
        bytes.push(value.reg0 % 16); // Register encoded mod 16

        // Encode immediate
        let x_bytes = value.imm0.to_le_bytes();
        bytes.extend_from_slice(&x_bytes[..value.imm0.len() as usize]);
        bytes
    }
}

impl From<&[u8]> for RI {
    fn from(bytes: &[u8]) -> Self {
        if bytes == [0] || bytes.is_empty() {
            return Default::default();
        }

        RI {
            reg0: (bytes[0] % 16).min(12),
            imm0: u64::read_imm(&bytes[1..]),
        }
    }
}

#[test]
fn test_ri_encoding() {
    let bytes = vec![7, 0, 0, 2];
    let decoded = RI::from(bytes.as_ref());
    assert_eq!(decoded.reg0, 7);
    assert_eq!(decoded.imm0, 0x020000);

    let encoded: Vec<u8> = decoded.into();
    assert_eq!(encoded, bytes);
}
