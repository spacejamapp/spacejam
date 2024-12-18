use crate::format::{RII, RIO};

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
        let rii = RII::from(bytes);
        RIO {
            reg0: rii.reg0,
            imm0: rii.imm0,
            off0: rii.imm1,
        }
    }
}
