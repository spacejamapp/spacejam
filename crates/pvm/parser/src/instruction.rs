//! The PVM instructions.

use crate::format::{self, *};
use core::ops::Range;

include!(concat!(env!("OUT_DIR"), "/instruction.rs"));

/// Information about the instruction
#[derive(Debug, Clone, Default, Copy)]
pub enum InstructionType {
    /// General operations
    #[default]
    General,

    /// Function call operations
    Call(u64),

    /// Dynamic jump operations
    DynamicJump,

    /// Static jump operations
    StaticJump(u64),

    /// Memory operations
    Memory,
}

/// Information about the instruction
#[derive(Debug, Clone, Default)]
pub struct InstructionInfo {
    /// The type of the instruction
    pub ty: InstructionType,

    /// The range of the instruction
    pub range: Range<usize>,

    /// Input registers
    pub input: Vec<u8>,

    /// Output registers
    pub output: Vec<u8>,
}

impl InstructionInfo {
    /// Check if the instruction is a termination instruction.
    pub fn is_termination(&self) -> bool {
        matches!(
            self.ty,
            InstructionType::StaticJump(_)
                | InstructionType::DynamicJump
                | InstructionType::Call(_)
        )
    }

    /// Check if the instruction is a memory instruction.
    pub fn is_memory(&self) -> bool {
        matches!(self.ty, InstructionType::Memory)
    }
}

impl Instruction {
    /// Get the information about the instruction
    pub fn info(&self, range: Range<usize>) -> InstructionInfo {
        match self {
            Instruction::Add32(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::Add64(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::AddImm32(format::RRI {
                reg0,
                reg1,
                imm0: _,
            }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::AddImm64(format::RRI {
                reg0,
                reg1,
                imm0: _,
            }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::And(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::AndImm(format::RRI {
                reg0,
                reg1,
                imm0: _,
            }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::AndInv(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::BranchEq(format::RRO { reg0, reg1, off0 }) => InstructionInfo {
                ty: InstructionType::StaticJump((range.start as i64 + *off0 as i64) as u64),
                range,
                input: vec![*reg0, *reg1],
                output: vec![],
            },
            Instruction::BranchEqImm(format::RIO {
                reg0,
                off0,
                imm0: _,
            }) => InstructionInfo {
                ty: InstructionType::StaticJump((range.start as i64 + *off0 as i64) as u64),
                range,
                input: vec![*reg0],
                output: vec![],
            },
            Instruction::BranchGeS(format::RRO { reg0, reg1, off0 }) => InstructionInfo {
                ty: InstructionType::StaticJump((range.start as i64 + *off0 as i64) as u64),
                range,
                input: vec![*reg0, *reg1],
                output: vec![],
            },
            Instruction::BranchGeSImm(format::RIO {
                reg0,
                off0,
                imm0: _,
            }) => InstructionInfo {
                ty: InstructionType::StaticJump((range.start as i64 + *off0 as i64) as u64),
                range,
                input: vec![*reg0],
                output: vec![],
            },
            Instruction::BranchGeU(format::RRO { reg0, reg1, off0 }) => InstructionInfo {
                ty: InstructionType::StaticJump((range.start as i64 + *off0 as i64) as u64),
                range,
                input: vec![*reg0, *reg1],
                output: vec![],
            },
            Instruction::BranchGeUImm(format::RIO { reg0, off0, imm0 }) => InstructionInfo {
                ty: InstructionType::StaticJump((range.start as i64 + *off0 as i64) as u64),
                range,
                input: vec![*reg0],
                output: vec![],
            },
            Instruction::BranchGtSImm(format::RIO { reg0, off0, imm0 }) => InstructionInfo {
                ty: InstructionType::StaticJump((range.start as i64 + *off0 as i64) as u64),
                range,
                input: vec![*reg0],
                output: vec![],
            },
            Instruction::BranchGtUImm(format::RIO { reg0, off0, imm0 }) => InstructionInfo {
                ty: InstructionType::StaticJump((range.start as i64 + *off0 as i64) as u64),
                range,
                input: vec![*reg0],
                output: vec![],
            },
            Instruction::BranchLeSImm(format::RIO { reg0, off0, imm0 }) => InstructionInfo {
                ty: InstructionType::StaticJump((range.start as i64 + *off0 as i64) as u64),
                range,
                input: vec![*reg0],
                output: vec![],
            },
            Instruction::BranchLeUImm(format::RIO { reg0, off0, imm0 }) => InstructionInfo {
                ty: InstructionType::StaticJump((range.start as i64 + *off0 as i64) as u64),
                range,
                input: vec![*reg0],
                output: vec![],
            },
            Instruction::BranchLtS(format::RRO { reg0, reg1, off0 }) => InstructionInfo {
                ty: InstructionType::StaticJump((range.start as i64 + *off0 as i64) as u64),
                range,
                input: vec![*reg0, *reg1],
                output: vec![],
            },
            Instruction::BranchLtSImm(format::RIO { reg0, off0, imm0 }) => InstructionInfo {
                ty: InstructionType::StaticJump((range.start as i64 + *off0 as i64) as u64),
                range,
                input: vec![*reg0],
                output: vec![],
            },
            Instruction::BranchLtU(format::RRO { reg0, reg1, off0 }) => InstructionInfo {
                ty: InstructionType::StaticJump((range.start as i64 + *off0 as i64) as u64),
                range,
                input: vec![*reg0, *reg1],
                output: vec![],
            },
            Instruction::BranchLtUImm(format::RIO { reg0, off0, imm0 }) => InstructionInfo {
                ty: InstructionType::StaticJump((range.start as i64 + *off0 as i64) as u64),
                range,
                input: vec![*reg0],
                output: vec![],
            },
            Instruction::BranchNe(format::RRO { reg0, reg1, off0 }) => InstructionInfo {
                ty: InstructionType::StaticJump((range.start as i64 + *off0 as i64) as u64),
                range,
                input: vec![*reg0, *reg1],
                output: vec![],
            },
            Instruction::BranchNeImm(format::RIO { reg0, off0, imm0 }) => InstructionInfo {
                ty: InstructionType::StaticJump((range.start as i64 + *off0 as i64) as u64),
                range,
                input: vec![*reg0],
                output: vec![],
            },
            Instruction::CmovIz(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::CmovIzImm(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::CmovNz(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::CmovNzImm(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::CountSetBits32(format::RR { reg0, reg1 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::CountSetBits64(format::RR { reg0, reg1 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::DivU32(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::DivU64(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::DivS32(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::DivS64(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::Ecalli(format::I { imm0 }) => InstructionInfo {
                ty: InstructionType::StaticJump(range.start as u64),
                range,
                input: vec![],
                output: vec![],
            },
            Instruction::Fallthrough => InstructionInfo {
                ty: InstructionType::StaticJump(range.end as u64),
                range,
                input: vec![],
                output: vec![],
            },
            Instruction::Jump(format::O { off0 }) => InstructionInfo {
                ty: InstructionType::StaticJump((range.start as i64 + *off0 as i64) as u64),
                range,
                input: vec![],
                output: vec![],
            },
            Instruction::JumpInd(format::RI { reg0, imm0 }) => InstructionInfo {
                ty: InstructionType::DynamicJump,
                range,
                input: vec![*reg0],
                output: vec![],
            },
            Instruction::LeadingZeroBits32(format::RR { reg0, reg1 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::LeadingZeroBits64(format::RR { reg0, reg1 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::LoadI8(format::RI { reg0, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![],
                output: vec![*reg0],
            },
            Instruction::LoadI16(format::RI { reg0, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![],
                output: vec![*reg0],
            },
            Instruction::LoadI32(format::RI { reg0, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![],
                output: vec![*reg0],
            },
            Instruction::LoadImm(format::RI { reg0, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![],
                output: vec![*reg0],
            },
            Instruction::LoadImm64(format::REI { reg0, eimm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![],
                output: vec![*reg0],
            },
            Instruction::LoadImmJump(format::RIO { reg0, off0, imm0 }) => InstructionInfo {
                ty: InstructionType::Call((range.start as i64 + *off0 as i64) as u64),
                range,
                input: vec![],
                output: vec![*reg0],
            },
            Instruction::LoadImmJumpInd(format::RRII {
                reg0,
                reg1,
                imm0,
                imm1,
            }) => InstructionInfo {
                ty: InstructionType::DynamicJump,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::LoadIndI8(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::LoadIndU8(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::LoadIndU16(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::LoadIndI16(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::LoadIndU32(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::LoadIndI32(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::LoadIndU64(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::LoadU8(format::RI { reg0, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![],
                output: vec![*reg0],
            },
            Instruction::LoadU16(format::RI { reg0, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![],
                output: vec![*reg0],
            },
            Instruction::LoadU32(format::RI { reg0, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![],
                output: vec![*reg0],
            },
            Instruction::LoadU64(format::RI { reg0, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![],
                output: vec![*reg0],
            },
            Instruction::Max(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::MaxU(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::Min(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::MinU(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::MoveReg(format::RR { reg0, reg1 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::Mul32(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::Mul64(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::MulImm32(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::MulImm64(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::MulUpperSS(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::MulUpperUU(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::MulUpperSU(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::NegAddImm32(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::NegAddImm64(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::Or(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::OrImm(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::OrInv(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::RemU32(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::RemU64(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::RemS32(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::RemS64(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::ReverseBytes(format::RR { reg0, reg1 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::RotL32(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::RotL64(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::RotR32(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::RotR32Imm(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::RotR32ImmAlt(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::RotR64(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::RotR64Imm(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::RotR64ImmAlt(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::Sbrk(format::RR { reg0, reg1 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::SetGtSImm(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::SetGtUImm(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::SetLtSImm(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::SetLtUImm(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::SetLtU(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::SetLtS(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::SharR32(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::SharR64(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::SharRImm32(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::SharRImm64(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::SharRImmAlt32(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::SharRImmAlt64(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::ShloL32(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::ShloL64(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::ShloLImm32(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::ShloLImm64(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::ShloLImmAlt32(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::ShloLImmAlt64(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::ShloR32(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::ShloR64(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::ShloRImm32(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::ShloRImm64(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::ShloRImmAlt32(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::ShloRImmAlt64(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::SignExtend8(format::RR { reg0, reg1 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::SignExtend16(format::RR { reg0, reg1 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::StoreU8(format::RI { reg0, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![*reg0],
                output: vec![],
            },
            Instruction::StoreU16(format::RI { reg0, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![*reg0],
                output: vec![],
            },
            Instruction::StoreU32(format::RI { reg0, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![*reg0],
                output: vec![],
            },
            Instruction::StoreU64(format::RI { reg0, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![*reg0],
                output: vec![],
            },
            Instruction::StoreImmU8(format::II { imm0, imm1 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![],
                output: vec![],
            },
            Instruction::StoreImmU16(format::II { imm0, imm1 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![],
                output: vec![],
            },
            Instruction::StoreImmU32(format::II { imm0, imm1 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![],
                output: vec![],
            },
            Instruction::StoreImmU64(format::II { imm0, imm1 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![],
                output: vec![],
            },
            Instruction::StoreImmIndU8(format::RII { reg0, imm0, imm1 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![*reg0],
                output: vec![],
            },
            Instruction::StoreImmIndU16(format::RII { reg0, imm0, imm1 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![*reg0],
                output: vec![],
            },
            Instruction::StoreImmIndU32(format::RII { reg0, imm0, imm1 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![*reg0],
                output: vec![],
            },
            Instruction::StoreImmIndU64(format::RII { reg0, imm0, imm1 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![*reg0],
                output: vec![],
            },
            Instruction::StoreIndU8(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![*reg0, *reg1],
                output: vec![],
            },
            Instruction::StoreIndU16(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![*reg0, *reg1],
                output: vec![],
            },
            Instruction::StoreIndU32(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![*reg0, *reg1],
                output: vec![],
            },
            Instruction::StoreIndU64(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::Memory,
                range,
                input: vec![*reg0, *reg1],
                output: vec![],
            },
            Instruction::Sub32(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::Sub64(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::Trap => InstructionInfo {
                ty: InstructionType::StaticJump(range.end as u64),
                range,
                input: vec![],
                output: vec![],
            },
            Instruction::TrailingZeroBits32(format::RR { reg0, reg1 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::TrailingZeroBits64(format::RR { reg0, reg1 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::Xnor(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::Xor(format::RRR { reg0, reg1, reg2 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg0, *reg1],
                output: vec![*reg2],
            },
            Instruction::XorImm(format::RRI { reg0, reg1, imm0 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
            Instruction::ZeroExtend16(format::RR { reg0, reg1 }) => InstructionInfo {
                ty: InstructionType::General,
                range,
                input: vec![*reg1],
                output: vec![*reg0],
            },
        }
    }
}

impl Instruction {
    /// Check if the instruction is a termination instruction.
    pub fn is_termination(&self) -> bool {
        matches!(
            self,
            Instruction::Trap
                | Instruction::Fallthrough
                | Instruction::Jump(_)
                | Instruction::JumpInd(_)
                | Instruction::LoadImmJump(_)
                | Instruction::LoadImmJumpInd(_)
                | Instruction::BranchEq(_)
                | Instruction::BranchNe(_)
                | Instruction::BranchGeU(_)
                | Instruction::BranchGeS(_)
                | Instruction::BranchLtU(_)
                | Instruction::BranchLtS(_)
                | Instruction::BranchEqImm(_)
                | Instruction::BranchNeImm(_)
                | Instruction::BranchGeUImm(_)
                | Instruction::BranchGeSImm(_)
                | Instruction::BranchLtUImm(_)
                | Instruction::BranchLtSImm(_)
                | Instruction::BranchLeUImm(_)
                | Instruction::BranchLeSImm(_)
                | Instruction::BranchGtUImm(_)
                | Instruction::BranchGtSImm(_)
        )
    }

    /// Check if the instruction is a memory operation.
    pub fn is_memory_op(&self) -> bool {
        matches!(
            self,
            Instruction::StoreImmU16(_)
                | Instruction::StoreImmU32(_)
                | Instruction::StoreImmU64(_)
                | Instruction::StoreImmU8(_)
                | Instruction::StoreImmIndU16(_)
                | Instruction::StoreImmIndU32(_)
                | Instruction::StoreImmIndU64(_)
                | Instruction::StoreImmIndU8(_)
                | Instruction::StoreIndU8(_)
                | Instruction::StoreIndU16(_)
                | Instruction::StoreIndU32(_)
                | Instruction::StoreIndU64(_)
                | Instruction::StoreU16(_)
                | Instruction::StoreU32(_)
                | Instruction::StoreU64(_)
                | Instruction::StoreU8(_)
                | Instruction::LoadIndI16(_)
                | Instruction::LoadIndI32(_)
                | Instruction::LoadIndI8(_)
                | Instruction::LoadIndU16(_)
                | Instruction::LoadIndU32(_)
                | Instruction::LoadIndU64(_)
                | Instruction::LoadIndU8(_)
                | Instruction::LoadI16(_)
                | Instruction::LoadI32(_)
                | Instruction::LoadI8(_)
                | Instruction::LoadU16(_)
                | Instruction::LoadU32(_)
                | Instruction::LoadU64(_)
                | Instruction::LoadU8(_)
        )
    }
}
