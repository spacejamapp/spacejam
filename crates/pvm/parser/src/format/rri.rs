//! The RRI instruction format.

use crate::format::{ISA, RRI};

impl From<RRI> for Vec<u8> {
    fn from(value: RRI) -> Self {
        let mut bytes = Vec::with_capacity(5);
        // Two registers packed into first byte
        bytes.push(((value.reg1 % 16) << 4) | (value.reg0 % 16));

        // Encode immediate
        let x_len = value.imm0.len();
        let x_bytes = value.imm0.to_le_bytes();
        bytes.extend_from_slice(&x_bytes[..x_len as usize]);

        bytes
    }
}

impl From<&[u8]> for RRI {
    fn from(bytes: &[u8]) -> Self {
        if bytes == [0] || bytes.is_empty() {
            return Default::default();
        }

        let imm0 = if bytes.len() == 1 {
            0
        } else {
            u64::read_imm(&bytes[1..])
        };

        RRI {
            reg0: (bytes[0] & 0x0f).min(12),
            reg1: (bytes[0] >> 4).min(12),
            imm0,
        }
    }
}

#[test]
fn rri_encoding() {
    let bytes = vec![121, 2];
    let decoded = RRI::from(bytes.as_ref());
    let encoded: Vec<u8> = decoded.into();
    assert_eq!(bytes, encoded);
}
