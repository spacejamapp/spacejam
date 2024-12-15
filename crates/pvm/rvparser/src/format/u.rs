//! RISC-V U-type instruction

use crate::format::{self, Format};

/// RISC-V U-type instruction
#[derive(Debug, Clone, Copy)]
pub struct UType {
    pub imm_31_12: u32,
    pub rd: u8,
}

impl Format for UType {
    const OPCODE: u8 = 0b0110111;
}

impl From<[u8; 4]> for UType {
    fn from(bytes: [u8; 4]) -> Self {
        let value = u32::from_le_bytes(bytes);
        Self {
            imm_31_12: format::extract_bits(value, 31, 12),
            rd: format::extract_bits(value, 11, 7) as u8,
        }
    }
}

impl From<UType> for [u8; 4] {
    fn from(instr: UType) -> Self {
        let mut value = 0u32;
        value |= (instr.imm_31_12) << 12;
        value |= (instr.rd as u32) << 7;
        value |= UType::OPCODE as u32;
        value.to_le_bytes()
    }
}
