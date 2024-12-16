//! RISC-V S-type instruction

use crate::format::{self, Format};

/// RISC-V S-type instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SType {
    /// imm\[11:5\]
    pub imm_11_5: u8,
    /// imm\[4:0\]
    pub imm_4_0: u8,
    /// rs1
    pub rs1: u8,
    /// rs2
    pub rs2: u8,
    /// funct3
    pub funct3: u8,
}

impl Format for SType {
    const OPCODE: u8 = 0b0100011;
}

impl From<u32> for SType {
    fn from(value: u32) -> Self {
        Self {
            imm_11_5: format::extract_bits(value, 31, 25) as u8,
            imm_4_0: format::extract_bits(value, 11, 7) as u8,
            rs2: format::extract_bits(value, 24, 20) as u8,
            rs1: format::extract_bits(value, 19, 15) as u8,
            funct3: format::extract_bits(value, 14, 12) as u8,
        }
    }
}

impl From<[u8; 4]> for SType {
    fn from(bytes: [u8; 4]) -> Self {
        Self::from(u32::from_le_bytes(bytes))
    }
}

impl From<SType> for u32 {
    fn from(instr: SType) -> Self {
        let mut value = 0u32;
        value |= (instr.imm_11_5 as u32) << 25;
        value |= (instr.imm_4_0 as u32) << 7;
        value |= (instr.rs2 as u32) << 20;
        value |= (instr.rs1 as u32) << 15;
        value |= (instr.funct3 as u32) << 12;
        value |= SType::OPCODE as u32;
        value
    }
}

impl From<SType> for [u8; 4] {
    fn from(instr: SType) -> Self {
        u32::from(instr).to_le_bytes()
    }
}
