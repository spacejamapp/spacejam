//! RISC-V machine code parser
use crate::instr::Instruction;
use anyhow::Result;

/// Parse a RISC-V instruction
pub fn parse(instr: [u8; 4]) -> Result<Instruction> {
    todo!()
    // Instr::try_from(instr)
}
