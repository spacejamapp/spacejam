//! RISC-V J-type instruction

use crate::format::{self, Format};

/// RISC-V J-type instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JType {
    /// imm\[20\]
    pub imm_20: u8,
    /// imm\[19:12\]
    pub imm_19_12: u8,
    /// imm\[11\]
    pub imm_11: u8,
    /// imm\[10:1\]
    pub imm_10_1: u16,
    /// rd
    pub rd: u8,
}

impl Format for JType {
    const OPCODE: u8 = 0b1101111;
}

impl From<u32> for JType {
    fn from(value: u32) -> Self {
        Self {
            imm_20: format::extract_bits(value, 31, 31) as u8,
            imm_19_12: format::extract_bits(value, 19, 12) as u8,
            imm_11: format::extract_bits(value, 20, 20) as u8,
            imm_10_1: format::extract_bits(value, 30, 21) as u16,
            rd: format::extract_bits(value, 11, 7) as u8,
        }
    }
}

impl From<[u8; 4]> for JType {
    fn from(bytes: [u8; 4]) -> Self {
        Self::from(u32::from_le_bytes(bytes))
    }
}

impl From<JType> for u32 {
    fn from(instr: JType) -> Self {
        let mut value = 0u32;
        value |= (instr.imm_20 as u32) << 31;
        value |= (instr.imm_19_12 as u32) << 12;
        value |= (instr.imm_11 as u32) << 20;
        value |= (instr.imm_10_1 as u32) << 21;
        value |= (instr.rd as u32) << 7;
        value |= JType::OPCODE as u32;
        value
    }
}

impl From<JType> for [u8; 4] {
    fn from(instr: JType) -> Self {
        u32::from(instr).to_le_bytes()
    }
}
