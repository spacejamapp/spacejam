//! Utility functions shared across the compiler

/// Check if a PVM instruction is terminating (ends basic block execution)
pub fn is_terminating_instruction(instruction: &parser::Instruction) -> bool {
    matches!(
        instruction,
        parser::Instruction::Trap
            | parser::Instruction::Fallthrough
            | parser::Instruction::Jump(_)
            | parser::Instruction::JumpInd(_)
            | parser::Instruction::LoadImmJump(_)
            | parser::Instruction::LoadImmJumpInd(_)
            | parser::Instruction::BranchEq(_)
            | parser::Instruction::BranchNe(_)
            | parser::Instruction::BranchGeU(_)
            | parser::Instruction::BranchGeS(_)
            | parser::Instruction::BranchLtU(_)
            | parser::Instruction::BranchLtS(_)
            | parser::Instruction::BranchEqImm(_)
            | parser::Instruction::BranchNeImm(_)
            | parser::Instruction::BranchGeUImm(_)
            | parser::Instruction::BranchGeSImm(_)
            | parser::Instruction::BranchLtUImm(_)
            | parser::Instruction::BranchLtSImm(_)
            | parser::Instruction::BranchLeUImm(_)
            | parser::Instruction::BranchLeSImm(_)
            | parser::Instruction::BranchGtUImm(_)
            | parser::Instruction::BranchGtSImm(_)
    )
}
