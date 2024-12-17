use crate::format::{Format, ISA, RRO};

impl Format for RRO {
    const MIN_LEN: usize = 2;
    const MAX_LEN: usize = 5;
}

impl TryFrom<&[u8]> for RRO {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < 3 {
            anyhow::bail!("Invalid byte length, expected at least 3 bytes");
        }

        Ok(RRO {
            reg0: (bytes[0] & 0x0f).min(12),
            reg1: (bytes[0] >> 4).min(12),
            off0: u32::read(&bytes[1..])?,
        })
    }
}

impl From<RRO> for Vec<u8> {
    fn from(value: RRO) -> Self {
        let mut bytes = vec![((value.reg1 << 4) | value.reg0)];
        bytes.extend(value.off0.bytes());
        bytes
    }
}
