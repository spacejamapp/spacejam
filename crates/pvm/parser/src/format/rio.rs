use crate::format::{Format, RII, RIO};

impl Format for RIO {
    const MIN_LEN: usize = 3;
    const MAX_LEN: usize = 9;
}

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

impl TryFrom<&[u8]> for RIO {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let rii = RII::try_from(bytes)?;

        Ok(RIO {
            reg0: rii.reg0,
            imm0: rii.imm0,
            off0: rii.imm1,
        })
    }
}
