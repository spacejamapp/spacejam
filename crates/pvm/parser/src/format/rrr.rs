use crate::format::RRR;

impl From<[u8; 2]> for RRR {
    fn from(bytes: [u8; 2]) -> Self {
        let byte0 = bytes[0];
        let byte1 = bytes[1];

        // Extract reg0 from lower 4 bits of byte0
        let reg0 = byte0 & 0x0F;

        // Extract reg1 from upper 4 bits of byte0
        let reg1 = (byte0 >> 4) & 0x0F;

        // reg2 is just byte1
        let reg2 = byte1;

        // Cap all registers at 12 per specification
        RRR {
            reg0: reg0.min(12),
            reg1: reg1.min(12),
            reg2: reg2.min(12),
        }
    }
}

impl From<RRR> for [u8; 2] {
    fn from(value: RRR) -> Self {
        // Cap register values at 12 per specification
        let r0 = value.reg0.min(12);
        let r1 = value.reg1.min(12);
        let r2 = value.reg2.min(12);

        // Encode byte0: r1 in upper 4 bits, r0 in lower 4 bits
        let byte0 = (r1 << 4) | r0;

        // byte1 is just r2
        let byte1 = r2;

        [byte0, byte1]
    }
}

#[test]
fn encoding() {
    let bytes = [0x87, 0x09];
    let decoded = RRR::from(bytes);
    let encoded: [u8; 2] = decoded.into();
    assert_eq!(encoded, bytes);
}
