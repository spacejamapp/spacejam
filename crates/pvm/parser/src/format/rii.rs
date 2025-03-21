//! The RII instruction format.

use crate::format::{ISA, RII};

impl From<RII> for Vec<u8> {
    fn from(value: RII) -> Self {
        let mut bytes = Vec::with_capacity(5);
        // Register encoded mod 16 in first byte
        bytes.push(value.reg0 % 16);

        // Encode first immediate
        let x_bytes = value.imm0.to_le_bytes();
        bytes.extend_from_slice(&x_bytes[..value.imm0.len() as usize]);

        // Encode second immediate
        let y_bytes = value.imm1.to_le_bytes();
        bytes.extend_from_slice(&y_bytes[..value.imm1.len() as usize]);

        bytes
    }
}

impl From<&[u8]> for RII {
    fn from(bytes: &[u8]) -> Self {
        if bytes == [0] || bytes.is_empty() {
            return Default::default();
        }

        let mid = (bytes.len() - 1).min(4) + 1;
        RII {
            reg0: bytes[0] % 16,
            imm0: u64::read_imm(&bytes[1..mid]),
            imm1: u64::read_imm(&bytes[mid..]),
        }
    }
}
