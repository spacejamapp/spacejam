//! RISC-V B-type instruction

use crate::format::{self, Format};

/// RISC-V B-type instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BType {
    /// imm\[12\]
    pub imm_12: u8,
    /// imm\[11\]
    pub imm_11: u8,
    /// imm\[10:5\]
    pub imm_10_5: u8,
    /// imm\[4:1\]
    pub imm_4_1: u8,
    pub rs1: u8,
    pub rs2: u8,
    pub funct3: u8,
}

impl Format for BType {
    const OPCODE: u8 = 0b1100011;
}

impl From<[u8; 4]> for BType {
    fn from(bytes: [u8; 4]) -> Self {
        let value = u32::from_le_bytes(bytes);
        Self {
            imm_12: format::extract_bits(value, 31, 31) as u8,
            imm_11: format::extract_bits(value, 7, 7) as u8,
            imm_10_5: format::extract_bits(value, 30, 25) as u8,
            imm_4_1: format::extract_bits(value, 11, 8) as u8,
            rs2: format::extract_bits(value, 24, 20) as u8,
            rs1: format::extract_bits(value, 19, 15) as u8,
            funct3: format::extract_bits(value, 14, 12) as u8,
        }
    }
}

impl From<BType> for [u8; 4] {
    fn from(instr: BType) -> Self {
        let mut value = 0u32;
        value |= (instr.imm_12 as u32) << 31;
        value |= (instr.imm_10_5 as u32) << 25;
        value |= (instr.imm_4_1 as u32) << 8;
        value |= (instr.imm_11 as u32) << 7;
        value |= (instr.rs2 as u32) << 20;
        value |= (instr.rs1 as u32) << 15;
        value |= (instr.funct3 as u32) << 12;
        value |= BType::OPCODE as u32;
        value.to_le_bytes()
    }
}
