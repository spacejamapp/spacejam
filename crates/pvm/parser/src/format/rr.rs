use crate::format::{Format, RR};

impl Format for RR {
    const MIN_LEN: usize = 1;
    const MAX_LEN: usize = 1;
}

impl From<u8> for RR {
    fn from(byte: u8) -> Self {
        RR {
            reg0: (byte & 0x0f).min(12),
            reg1: (byte >> 4).min(12),
        }
    }
}

impl TryFrom<&[u8]> for RR {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::MIN_LEN {
            anyhow::bail!(
                "Insufficient bytes for RR format, expected at least {}",
                Self::MIN_LEN
            );
        }

        let bytes: [u8; 1] = bytes.try_into()?;
        Ok(Self::from(bytes[0]))
    }
}

impl From<RR> for u8 {
    fn from(value: RR) -> Self {
        (value.reg1 << 4) | value.reg0
    }
}

#[test]
fn encoding() {
    let bytes = 0x87;
    let decoded = RR::from(bytes);
    let encoded: u8 = decoded.into();
    assert_eq!(encoded, bytes);
}
