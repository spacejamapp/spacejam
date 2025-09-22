//! The RIO instruction format.

use crate::format::{ISA, RII, RIO};

impl From<RIO> for Vec<u8> {
    fn from(value: RIO) -> Self {
        let rii = RII {
            reg0: value.reg0,
            imm0: value.imm0,
            imm1: value.off0 as u64,
        };

        rii.into()
    }
}

impl From<&[u8]> for RIO {
    fn from(bytes: &[u8]) -> Self {
        if bytes == [0] || bytes.is_empty() {
            return Default::default();
        }

        let mid = (bytes[0] / 16 % 8).min(4) as usize + 1;
        let offset_bytes = &bytes[mid..];
        let offset_len = offset_bytes.len().min(4);

        // Read the raw offset value without sign extension
        let (raw_offset, _) = u64::read(&offset_bytes[..offset_len]);

        // Sign-extend the offset based on its actual length
        let signed_offset = raw_offset.sign_extend(offset_len) as i32;

        RIO {
            reg0: (bytes[0] % 16).min(12),
            imm0: u64::read_imm(&bytes[1..mid]),
            off0: signed_offset,
        }
    }
}
