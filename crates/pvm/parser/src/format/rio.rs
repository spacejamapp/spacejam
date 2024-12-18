use crate::format::{ISA, RII, RIO};

impl From<RIO> for Vec<u8> {
    fn from(value: RIO) -> Self {
        let rii = RII {
            reg0: value.reg0,
            imm0: value.imm0,
            imm1: value.off0,
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
        RIO {
            reg0: (bytes[0] % 16).min(12),
            imm0: u32::read_imm(&bytes[1..mid]),
            off0: u32::read(&bytes[mid..]),
        }
    }
}
