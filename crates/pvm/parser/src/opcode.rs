//! The opcode definitions.

include!(concat!(env!("OUT_DIR"), "/opcode.rs"));

impl From<Opcode> for u8 {
    fn from(opcode: Opcode) -> Self {
        opcode as u8
    }
}
