//! RISC-V instructions.

include!(concat!(env!("OUT_DIR"), "/instr.rs"));

impl TryFrom<[u8; 4]> for Instruction {
    type Error = anyhow::Error;

    fn try_from(bits: [u8; 4]) -> anyhow::Result<Self> {
        Self::try_from(u32::from_le_bytes(bits))
    }
}
