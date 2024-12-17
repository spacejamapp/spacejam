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
