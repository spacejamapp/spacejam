use crate::format::{Format, RRR};

impl Format for RRR {
    const MIN_LEN: usize = 2;
    const MAX_LEN: usize = 2;
}

impl From<[u8; 2]> for RRR {
    fn from(bytes: [u8; 2]) -> Self {
        RRR {
            reg0: (bytes[0] & 0x0f).min(12),
            reg1: (bytes[0] >> 4).min(12),
            reg2: bytes[1].min(12),
        }
    }
}

impl TryFrom<&[u8]> for RRR {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::MIN_LEN {
            anyhow::bail!(
                "Insufficient bytes for RRR format, expected at least {}",
                Self::MIN_LEN
            );
        }

        let bytes: [u8; 2] = bytes.try_into()?;
        Ok(Self::from(bytes))
    }
}

impl From<RRR> for [u8; 2] {
    fn from(value: RRR) -> Self {
        [((value.reg1 << 4) | value.reg0), value.reg2.min(12)]
    }
}

#[test]
fn encoding() {
    let bytes = [0x87, 0x09];
    let decoded = RRR::from(bytes);
    let encoded: [u8; 2] = decoded.into();
    assert_eq!(encoded, bytes);
}
