//! The RRR instruction format.

use crate::format::RRR;

impl From<[u8; 2]> for RRR {
    fn from(bytes: [u8; 2]) -> Self {
        RRR {
            reg0: (bytes[0] & 0x0f).min(12),
            reg1: (bytes[0] >> 4).min(12),
            reg2: bytes[1].min(12),
        }
    }
}

impl From<&[u8]> for RRR {
    fn from(bytes: &[u8]) -> Self {
        if bytes.is_empty() || bytes == [0] {
            return Default::default();
        }

        let mut source = [0u8; 2];
        if bytes.len() > 1 {
            source.copy_from_slice(&bytes[0..2]);
        } else {
            source[0] = bytes[0];
        }

        Self::from(source)
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
