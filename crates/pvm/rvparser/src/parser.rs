//! RISC-V machine code parser
use crate::instr::Instr;
use anyhow::Result;

/// Parse a RISC-V instruction
pub fn parse(instr: [u8; 4]) -> Result<Instr> {
    Instr::try_from(instr)
}
