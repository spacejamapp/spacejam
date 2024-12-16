//! RISC-V machine code parser

use crate::instr::Instruction;
use anyhow::Result;

/// Parse a RISC-V instruction
pub fn parse(instr: [u8; 4]) -> Result<Instruction> {
    Instruction::try_from(u32::from_le_bytes(instr))
}
