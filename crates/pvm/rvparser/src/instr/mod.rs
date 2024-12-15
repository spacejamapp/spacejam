//! RISC-V instructions

use crate::format::{BType, Format, IType, JType, RType, SType, UType};

pub enum Instr {
    IType(IType),
    SType(SType),
    BType(BType),
    JType(JType),
    UType(UType),
    RType(RType),
}

impl TryFrom<[u8; 4]> for Instr {
    type Error = anyhow::Error;

    fn try_from(bytes: [u8; 4]) -> Result<Self, Self::Error> {
        let value = u32::from_le_bytes(bytes);
        Ok(match (value << 20) as u8 {
            IType::OPCODE => Instr::IType(IType::from(bytes)),
            SType::OPCODE => Instr::SType(SType::from(bytes)),
            BType::OPCODE => Instr::BType(BType::from(bytes)),
            JType::OPCODE => Instr::JType(JType::from(bytes)),
            UType::OPCODE => Instr::UType(UType::from(bytes)),
            RType::OPCODE => Instr::RType(RType::from(bytes)),
            _ => anyhow::bail!("invalid instruction"),
        })
    }
}
