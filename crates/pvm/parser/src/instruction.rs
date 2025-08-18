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
}
