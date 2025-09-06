//! The PVM instructions.

use crate::format::*;

include!(concat!(env!("OUT_DIR"), "/instruction.rs"));

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
