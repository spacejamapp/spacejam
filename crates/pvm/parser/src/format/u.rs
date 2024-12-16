//! RISC-V U-type instruction

use crate::format::{self, Format};

/// RISC-V U-type instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UType {
    /// imm\[31:12\]
    pub imm_31_12: u32,
    /// rd
    pub rd: u8,
}

impl Format for UType {
    const OPCODE: u8 = 0b0010111;
}

impl From<u32> for UType {
    fn from(value: u32) -> Self {
        Self {
            imm_31_12: format::extract_bits(value, 31, 12),
            rd: format::extract_bits(value, 11, 7) as u8,
        }
    }
}

impl From<[u8; 4]> for UType {
    fn from(bytes: [u8; 4]) -> Self {
        Self::from(u32::from_le_bytes(bytes))
    }
}

impl From<UType> for u32 {
    fn from(instr: UType) -> Self {
        let mut value = 0u32;
        value |= (instr.imm_31_12) << 12;
        value |= (instr.rd as u32) << 7;
        value |= UType::OPCODE as u32;
        value
    }
}

impl From<UType> for [u8; 4] {
    fn from(instr: UType) -> Self {
        u32::from(instr).to_le_bytes()
    }
}
