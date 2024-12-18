use crate::format::RR;

impl From<u8> for RR {
    fn from(byte: u8) -> Self {
        RR {
            reg0: (byte & 0x0f).min(12),
            reg1: (byte >> 4).min(12),
        }
    }
}

impl From<&[u8]> for RR {
    fn from(bytes: &[u8]) -> Self {
        if bytes.is_empty() {
            return Default::default();
        }

        Self::from(bytes[0])
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
