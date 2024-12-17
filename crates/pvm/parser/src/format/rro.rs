use crate::format::{ISA, O, RRO};

impl TryFrom<&[u8]> for RRO {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() <= 2 {
            anyhow::bail!("Invalid length");
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
        let o: Vec<u8> = O { off0: value.off0 }.into();
        bytes.extend(o);
        bytes
    }
}
