//! The RRO instruction format.

use crate::format::{ISA, RRI, RRO};

impl From<&[u8]> for RRO {
    fn from(bytes: &[u8]) -> Self {
        let rri = RRI::from(bytes);

        RRO {
            reg0: rri.reg0,
            reg1: rri.reg1,
            off0: rri.imm0,
        }
    }
}

impl From<RRO> for Vec<u8> {
    fn from(value: RRO) -> Self {
        let mut bytes = vec![((value.reg1 << 4) | value.reg0)];
        bytes.extend(value.off0.bytes());
        bytes
    }
}
