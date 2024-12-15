//! RISC-V I-type instruction

use crate::format::{self, Format};

/// RISC-V I-type instruction
#[derive(Debug, Clone, Copy)]
pub struct IType {
    pub imm_11_0: u16,
    pub rs1: u8,
    pub funct3: u8,
    pub rd: u8,
}

impl Format for IType {
    const OPCODE: u8 = 0b0010011;
}

impl From<[u8; 4]> for IType {
    fn from(bytes: [u8; 4]) -> Self {
        let value = u32::from_le_bytes(bytes);
        Self {
            imm_11_0: format::extract_bits(value, 31, 20) as u16,
            rs1: format::extract_bits(value, 19, 15) as u8,
            funct3: format::extract_bits(value, 14, 12) as u8,
            rd: format::extract_bits(value, 11, 7) as u8,
        }
    }
}

impl From<IType> for [u8; 4] {
    fn from(instr: IType) -> Self {
        let mut value = 0u32;
        value |= (instr.imm_11_0 as u32) << 20;
        value |= (instr.rs1 as u32) << 15;
        value |= (instr.funct3 as u32) << 12;
        value |= (instr.rd as u32) << 7;
        value |= IType::OPCODE as u32;
        value.to_le_bytes()
    }
}
