//! RISC-V R-type instruction

use crate::format::{self, Format};

/// RISC-V R-type instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RType {
    /// The funct7 field
    pub funct7: u8,
    /// The rs2 field
    pub rs2: u8,
    /// The rs1 field
    pub rs1: u8,
    /// The funct3 field
    pub funct3: u8,
    /// The rd field
    pub rd: u8,
}

impl Format for RType {
    const OPCODE: u8 = 0b0110011;
}

impl From<[u8; 4]> for RType {
    fn from(bytes: [u8; 4]) -> Self {
        let value = u32::from_le_bytes(bytes);
        Self {
            funct7: format::extract_bits(value, 31, 25) as u8,
            rs2: format::extract_bits(value, 24, 20) as u8,
            rs1: format::extract_bits(value, 19, 15) as u8,
            funct3: format::extract_bits(value, 14, 12) as u8,
            rd: format::extract_bits(value, 11, 7) as u8,
        }
    }
}

impl From<RType> for [u8; 4] {
    fn from(instr: RType) -> Self {
        let mut value = 0u32;
        value |= (instr.funct7 as u32) << 25;
        value |= (instr.rs2 as u32) << 20;
        value |= (instr.rs1 as u32) << 15;
        value |= (instr.funct3 as u32) << 12;
        value |= (instr.rd as u32) << 7;
        value |= RType::OPCODE as u32;
        value.to_le_bytes()
    }
}
