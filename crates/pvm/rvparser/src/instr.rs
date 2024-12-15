use anyhow::Result;

/// RISC-V instruction
pub trait Instruction: Sized {
    /// Parse the instruction from the given 32-bit data
    fn parse(data: [u8; 4]) -> Result<Self>;
}
