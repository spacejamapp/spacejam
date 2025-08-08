//! Control flow translation

use crate::Translator;
use cranelift::prelude::*;
use parser::{Instruction, Visitor};

impl<'a, 'b> Translator<'a, 'b> {
    /// Check if the program has any control flow instructions
    pub fn has_control_flow(
        &self,
        blob: &parser::program::ProgramBlob,
    ) -> Result<bool, anyhow::Error> {
        let mut reader = blob.reader();

        while !reader.eof() {
            let instruction_offset = reader.read()?;
            let instruction = &instruction_offset.value;

            match instruction {
                Instruction::Jump(_)
                | Instruction::JumpInd(_)
                | Instruction::LoadImmJump(_)
                | Instruction::LoadImmJumpInd(_)
                | Instruction::BranchEq(_)
                | Instruction::BranchNe(_)
                | Instruction::BranchLtU(_)
                | Instruction::BranchLtS(_)
                | Instruction::BranchGeU(_)
                | Instruction::BranchGeS(_)
                | Instruction::BranchEqImm(_)
                | Instruction::BranchNeImm(_)
                | Instruction::BranchLtUImm(_)
                | Instruction::BranchLtSImm(_)
                | Instruction::BranchGeUImm(_)
                | Instruction::BranchGeSImm(_)
                | Instruction::BranchLeUImm(_)
                | Instruction::BranchLeSImm(_)
                | Instruction::BranchGtUImm(_)
                | Instruction::BranchGtSImm(_) => {
                    return Ok(true);
                }
                _ => {}
            }
        }

        Ok(false)
    }

    /// Pass 1: Analyze the program to identify all branch targets and basic block boundaries
    pub fn analyze_control_flow(
        &mut self,
        blob: &parser::program::ProgramBlob,
    ) -> Result<(), anyhow::Error> {
        let mut reader = blob.reader();

        // Always start at offset 0
        self.branch_targets.insert(0);

        while !reader.eof() {
            let instruction_offset = reader.read()?;
            let current_pc = instruction_offset.range.start;
            let instruction = &instruction_offset.value;

            // Check if this instruction is a control flow instruction
            match instruction {
                Instruction::Jump(format) => {
                    let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                    self.branch_targets.insert(target_offset);
                    // Next instruction after jump is also a basic block start
                    self.branch_targets.insert(instruction_offset.range.end);
                }
                Instruction::JumpInd(_) => {
                    // For indirect jumps, add all jump table targets as basic block boundaries
                    for &jump_target in &blob.jump_table {
                        self.branch_targets.insert(jump_target as usize);
                    }
                    // Next instruction after jump is also a basic block boundary
                    self.branch_targets.insert(instruction_offset.range.end);
                }
                Instruction::LoadImmJump(format) => {
                    let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                    self.branch_targets.insert(target_offset);
                    // Next instruction after jump is also a basic block start
                    self.branch_targets.insert(instruction_offset.range.end);
                }
                Instruction::LoadImmJumpInd(_) => {
                    // For indirect jumps, add all jump table targets as basic block boundaries
                    for &jump_target in &blob.jump_table {
                        self.branch_targets.insert(jump_target as usize);
                    }
                    // Next instruction after jump is also a basic block boundary
                    self.branch_targets.insert(instruction_offset.range.end);
                }
                Instruction::BranchEq(format)
                | Instruction::BranchNe(format)
                | Instruction::BranchLtU(format)
                | Instruction::BranchLtS(format)
                | Instruction::BranchGeU(format)
                | Instruction::BranchGeS(format) => {
                    let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                    self.branch_targets.insert(target_offset);
                    // Next instruction after branch is also a basic block start (fall-through path)
                    self.branch_targets.insert(instruction_offset.range.end);
                }
                Instruction::BranchEqImm(format)
                | Instruction::BranchNeImm(format)
                | Instruction::BranchLtUImm(format)
                | Instruction::BranchLtSImm(format)
                | Instruction::BranchGeUImm(format)
                | Instruction::BranchGeSImm(format)
                | Instruction::BranchLeUImm(format)
                | Instruction::BranchLeSImm(format)
                | Instruction::BranchGtUImm(format)
                | Instruction::BranchGtSImm(format) => {
                    let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                    self.branch_targets.insert(target_offset);
                    // Next instruction after branch is also a basic block start (fall-through path)
                    self.branch_targets.insert(instruction_offset.range.end);
                }
                _ => {} // Not a control flow instruction
            }
        }

        Ok(())
    }

    /// Execute an instruction with proper control flow handling
    /// Returns true if the block still needs fallthrough, false if it's terminated
    pub fn visit_with_control_flow(
        &mut self,
        instruction: Instruction,
        current_pc: usize,
        next_pc: usize,
    ) -> Result<bool, anyhow::Error> {
        match instruction {
            // Control flow instructions need special handling
            Instruction::Jump(format) => {
                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];

                // Set PC for the target block
                let target_pc = self.builder.ins().iconst(types::I64, target_offset as i64);
                self.builder.def_var(self.pc, target_pc);

                // TODO: Pass context parameter to the target block
                self.builder.ins().jump(target_block, &[]);
                Ok(false) // Block is terminated, no fallthrough needed
            }
            Instruction::JumpInd(format) => {
                // Indirect jump: PC = reg0 + immediate
                let reg0_var = self.registers[&format.reg0];
                let reg0_val = self.builder.use_var(reg0_var);
                let offset = self.builder.ins().iconst(types::I64, format.imm0 as i64);
                let target_addr = self.builder.ins().iadd(reg0_val, offset);

                // Implement dynamic jump using Cranelift switch instruction
                self.emit_dynamic_jump(target_addr, current_pc, next_pc)?;
                Ok(false) // Block is terminated
            }
            Instruction::LoadImmJump(format) => {
                // Load immediate into register and then jump
                // First load the immediate
                let reg_var = self.registers[&format.reg0];
                let imm_val = self.builder.ins().iconst(types::I64, format.imm0 as i64);
                self.builder.def_var(reg_var, imm_val);

                // Then perform the jump
                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];

                // Set PC for the target block
                let target_pc = self.builder.ins().iconst(types::I64, target_offset as i64);
                self.builder.def_var(self.pc, target_pc);

                self.builder.ins().jump(target_block, &[]);
                Ok(false) // Block is terminated
            }
            Instruction::LoadImmJumpInd(format) => {
                // Load immediate into first register and jump indirect to second register + immediate
                
                // Handle same-register case: need to read reg1 value BEFORE storing to reg0
                let reg1_var = self.registers[&format.reg1];
                let reg1_val = self.builder.use_var(reg1_var);
                
                // Load the immediate into reg0
                let reg0_var = self.registers[&format.reg0];
                let imm0_val = self.builder.ins().iconst(types::I64, format.imm0 as i64);
                self.builder.def_var(reg0_var, imm0_val);

                // Compute jump target: reg1 + imm1 (using the value read before modifying reg0)
                let imm1_val = self.builder.ins().iconst(types::I64, format.imm1 as i64);
                let target_addr = self.builder.ins().iadd(reg1_val, imm1_val);

                // Use the hybrid dynamic jump dispatch
                self.emit_dynamic_jump(target_addr, current_pc, next_pc)?;
                Ok(false) // Block is terminated
            }
            Instruction::BranchEq(format) => {
                let reg0_var = self.registers[&format.reg0];
                let reg1_var = self.registers[&format.reg1];
                let reg0_val = self.builder.use_var(reg0_var);
                let reg1_val = self.builder.use_var(reg1_var);
                let condition = self.builder.ins().icmp(IntCC::Equal, reg0_val, reg1_val);

                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks.get(&target_offset)
                    .ok_or_else(|| anyhow::anyhow!("Branch target block not found at offset {}", target_offset))?;
                let fallthrough_block = self.basic_blocks.get(&next_pc)
                    .ok_or_else(|| anyhow::anyhow!("Fallthrough block not found at offset {}", next_pc))?;

                // Create PC values for both paths
                let target_pc = self.builder.ins().iconst(types::I64, target_offset as i64);
                let fallthrough_pc = self.builder.ins().iconst(types::I64, next_pc as i64);

                // Use select to set PC based on condition before branching
                let new_pc = self
                    .builder
                    .ins()
                    .select(condition, target_pc, fallthrough_pc);
                self.builder.def_var(self.pc, new_pc);

                self.builder
                    .ins()
                    .brif(condition, *target_block, &[], *fallthrough_block, &[]);
                Ok(false) // Block is terminated with conditional branch
            }
            Instruction::BranchNe(format) => {
                let reg0_var = self.registers[&format.reg0];
                let reg1_var = self.registers[&format.reg1];
                let reg0_val = self.builder.use_var(reg0_var);
                let reg1_val = self.builder.use_var(reg1_var);
                let condition = self.builder.ins().icmp(IntCC::NotEqual, reg0_val, reg1_val);

                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];
                let fallthrough_block = self.basic_blocks[&next_pc];

                self.builder
                    .ins()
                    .brif(condition, target_block, &[], fallthrough_block, &[]);
                Ok(false) // Block is terminated with conditional branch
            }
            Instruction::BranchEqImm(format) => {
                let reg0_var = self.registers[&format.reg0];
                let reg0_val = self.builder.use_var(reg0_var);
                let imm_val = self.builder.ins().iconst(types::I64, format.imm0 as i64);
                let condition = self.builder.ins().icmp(IntCC::Equal, reg0_val, imm_val);

                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];
                let fallthrough_block = self.basic_blocks[&next_pc];

                self.builder
                    .ins()
                    .brif(condition, target_block, &[], fallthrough_block, &[]);
                Ok(false) // Block is terminated with conditional branch
            }
            Instruction::BranchNeImm(format) => {
                let reg0_var = self.registers[&format.reg0];
                let reg0_val = self.builder.use_var(reg0_var);
                let imm_val = self.builder.ins().iconst(types::I64, format.imm0 as i64);
                let condition = self.builder.ins().icmp(IntCC::NotEqual, reg0_val, imm_val);

                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];
                let fallthrough_block = self.basic_blocks[&next_pc];

                self.builder
                    .ins()
                    .brif(condition, target_block, &[], fallthrough_block, &[]);
                Ok(false) // Block is terminated with conditional branch
            }
            // Signed comparison branches (register-register)
            Instruction::BranchLtS(format) => {
                let reg0_var = self.registers[&format.reg0];
                let reg1_var = self.registers[&format.reg1];
                let reg0_val = self.builder.use_var(reg0_var);
                let reg1_val = self.builder.use_var(reg1_var);
                let condition = self
                    .builder
                    .ins()
                    .icmp(IntCC::SignedLessThan, reg0_val, reg1_val);

                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];
                let fallthrough_block = self.basic_blocks[&next_pc];

                self.builder
                    .ins()
                    .brif(condition, target_block, &[], fallthrough_block, &[]);
                Ok(false) // Block is terminated with conditional branch
            }
            Instruction::BranchGeS(format) => {
                let reg0_var = self.registers[&format.reg0];
                let reg1_var = self.registers[&format.reg1];
                let reg0_val = self.builder.use_var(reg0_var);
                let reg1_val = self.builder.use_var(reg1_var);
                let condition =
                    self.builder
                        .ins()
                        .icmp(IntCC::SignedGreaterThanOrEqual, reg0_val, reg1_val);

                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];
                let fallthrough_block = self.basic_blocks[&next_pc];

                self.builder
                    .ins()
                    .brif(condition, target_block, &[], fallthrough_block, &[]);
                Ok(false) // Block is terminated with conditional branch
            }
            // Unsigned comparison branches (register-register)
            Instruction::BranchLtU(format) => {
                let reg0_var = self.registers[&format.reg0];
                let reg1_var = self.registers[&format.reg1];
                let reg0_val = self.builder.use_var(reg0_var);
                let reg1_val = self.builder.use_var(reg1_var);
                let condition =
                    self.builder
                        .ins()
                        .icmp(IntCC::UnsignedLessThan, reg0_val, reg1_val);

                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];
                let fallthrough_block = self.basic_blocks[&next_pc];

                self.builder
                    .ins()
                    .brif(condition, target_block, &[], fallthrough_block, &[]);
                Ok(false) // Block is terminated with conditional branch
            }
            Instruction::BranchGeU(format) => {
                let reg0_var = self.registers[&format.reg0];
                let reg1_var = self.registers[&format.reg1];
                let reg0_val = self.builder.use_var(reg0_var);
                let reg1_val = self.builder.use_var(reg1_var);
                let condition =
                    self.builder
                        .ins()
                        .icmp(IntCC::UnsignedGreaterThanOrEqual, reg0_val, reg1_val);

                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];
                let fallthrough_block = self.basic_blocks[&next_pc];

                self.builder
                    .ins()
                    .brif(condition, target_block, &[], fallthrough_block, &[]);
                Ok(false) // Block is terminated with conditional branch
            }
            // Signed immediate comparison branches
            Instruction::BranchLtSImm(format) => {
                let reg0_var = self.registers[&format.reg0];
                let reg0_val = self.builder.use_var(reg0_var);
                let imm_val = self.builder.ins().iconst(types::I64, format.imm0 as i64);
                let condition = self
                    .builder
                    .ins()
                    .icmp(IntCC::SignedLessThan, reg0_val, imm_val);

                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];
                let fallthrough_block = self.basic_blocks[&next_pc];

                self.builder
                    .ins()
                    .brif(condition, target_block, &[], fallthrough_block, &[]);
                Ok(false) // Block is terminated with conditional branch
            }
            Instruction::BranchGeSImm(format) => {
                let reg0_var = self.registers[&format.reg0];
                let reg0_val = self.builder.use_var(reg0_var);
                let imm_val = self.builder.ins().iconst(types::I64, format.imm0 as i64);
                let condition =
                    self.builder
                        .ins()
                        .icmp(IntCC::SignedGreaterThanOrEqual, reg0_val, imm_val);

                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];
                let fallthrough_block = self.basic_blocks[&next_pc];

                self.builder
                    .ins()
                    .brif(condition, target_block, &[], fallthrough_block, &[]);
                Ok(false) // Block is terminated with conditional branch
            }
            // Unsigned immediate comparison branches
            Instruction::BranchLtUImm(format) => {
                let reg0_var = self.registers[&format.reg0];
                let reg0_val = self.builder.use_var(reg0_var);
                let imm_val = self.builder.ins().iconst(types::I64, format.imm0 as i64);
                let condition = self
                    .builder
                    .ins()
                    .icmp(IntCC::UnsignedLessThan, reg0_val, imm_val);

                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];
                let fallthrough_block = self.basic_blocks[&next_pc];

                self.builder
                    .ins()
                    .brif(condition, target_block, &[], fallthrough_block, &[]);
                Ok(false) // Block is terminated with conditional branch
            }
            Instruction::BranchGeUImm(format) => {
                let reg0_var = self.registers[&format.reg0];
                let reg0_val = self.builder.use_var(reg0_var);
                let imm_val = self.builder.ins().iconst(types::I64, format.imm0 as i64);
                let condition =
                    self.builder
                        .ins()
                        .icmp(IntCC::UnsignedGreaterThanOrEqual, reg0_val, imm_val);

                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];
                let fallthrough_block = self.basic_blocks[&next_pc];

                self.builder
                    .ins()
                    .brif(condition, target_block, &[], fallthrough_block, &[]);
                Ok(false) // Block is terminated with conditional branch
            }
            // Additional immediate comparison branches (Le = Less or Equal, Gt = Greater Than)
            Instruction::BranchLeSImm(format) => {
                let reg0_var = self.registers[&format.reg0];
                let reg0_val = self.builder.use_var(reg0_var);
                let imm_val = self.builder.ins().iconst(types::I64, format.imm0 as i64);
                let condition =
                    self.builder
                        .ins()
                        .icmp(IntCC::SignedLessThanOrEqual, reg0_val, imm_val);

                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];
                let fallthrough_block = self.basic_blocks[&next_pc];

                self.builder
                    .ins()
                    .brif(condition, target_block, &[], fallthrough_block, &[]);
                Ok(false) // Block is terminated with conditional branch
            }
            Instruction::BranchLeUImm(format) => {
                let reg0_var = self.registers[&format.reg0];
                let reg0_val = self.builder.use_var(reg0_var);
                let imm_val = self.builder.ins().iconst(types::I64, format.imm0 as i64);
                let condition =
                    self.builder
                        .ins()
                        .icmp(IntCC::UnsignedLessThanOrEqual, reg0_val, imm_val);

                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];
                let fallthrough_block = self.basic_blocks[&next_pc];

                self.builder
                    .ins()
                    .brif(condition, target_block, &[], fallthrough_block, &[]);
                Ok(false) // Block is terminated with conditional branch
            }
            Instruction::BranchGtSImm(format) => {
                let reg0_var = self.registers[&format.reg0];
                let reg0_val = self.builder.use_var(reg0_var);
                let imm_val = self.builder.ins().iconst(types::I64, format.imm0 as i64);
                let condition =
                    self.builder
                        .ins()
                        .icmp(IntCC::SignedGreaterThan, reg0_val, imm_val);

                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];
                let fallthrough_block = self.basic_blocks[&next_pc];

                self.builder
                    .ins()
                    .brif(condition, target_block, &[], fallthrough_block, &[]);
                Ok(false) // Block is terminated with conditional branch
            }
            Instruction::BranchGtUImm(format) => {
                let reg0_var = self.registers[&format.reg0];
                let reg0_val = self.builder.use_var(reg0_var);
                let imm_val = self.builder.ins().iconst(types::I64, format.imm0 as i64);
                let condition =
                    self.builder
                        .ins()
                        .icmp(IntCC::UnsignedGreaterThan, reg0_val, imm_val);

                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];
                let fallthrough_block = self.basic_blocks[&next_pc];

                self.builder
                    .ins()
                    .brif(condition, target_block, &[], fallthrough_block, &[]);
                Ok(false) // Block is terminated with conditional branch
            }
            // Conditional move instructions
            Instruction::CmovIz(format) => {
                // Conditional move if zero: if reg1 == 0, reg2 = reg0 (following interpreter logic)
                let reg0_var = self.registers[&format.reg0];
                let reg1_var = self.registers[&format.reg1];
                let reg2_var = self.registers[&format.reg2];

                let reg0_val = self.builder.use_var(reg0_var); // source value
                let reg1_val = self.builder.use_var(reg1_var); // condition value
                let reg2_val = self.builder.use_var(reg2_var); // destination current value

                // Check if reg1 is zero (condition register)
                let zero = self.builder.ins().iconst(types::I64, 0);
                let is_zero = self.builder.ins().icmp(IntCC::Equal, reg1_val, zero);

                // Select between reg0 (if condition met) or current reg2 value (if condition not met)
                let new_val = self.builder.ins().select(is_zero, reg0_val, reg2_val);
                self.builder.def_var(reg2_var, new_val);

                Ok(true) // Not a control flow instruction, needs fallthrough
            }
            Instruction::CmovNz(format) => {
                // Conditional move if not zero: if reg1 != 0, reg2 = reg0 (following interpreter logic)
                let reg0_var = self.registers[&format.reg0];
                let reg1_var = self.registers[&format.reg1];
                let reg2_var = self.registers[&format.reg2];

                let reg0_val = self.builder.use_var(reg0_var); // source value
                let reg1_val = self.builder.use_var(reg1_var); // condition value
                let reg2_val = self.builder.use_var(reg2_var); // destination current value

                // Check if reg1 is not zero (condition register)
                let zero = self.builder.ins().iconst(types::I64, 0);
                let not_zero = self.builder.ins().icmp(IntCC::NotEqual, reg1_val, zero);

                // Select between reg0 (if condition met) or current reg2 value (if condition not met)
                let new_val = self.builder.ins().select(not_zero, reg0_val, reg2_val);
                self.builder.def_var(reg2_var, new_val);

                Ok(true) // Not a control flow instruction, needs fallthrough
            }
            Instruction::CmovIzImm(format) => {
                // Conditional move immediate if zero: if reg1 == 0, reg0 = imm
                let reg0_var = self.registers[&format.reg0];
                let reg1_var = self.registers[&format.reg1];

                let reg1_val = self.builder.use_var(reg1_var);
                let reg0_val = self.builder.use_var(reg0_var);
                let imm_val = self.builder.ins().iconst(types::I64, format.imm0 as i64);

                // Check if reg1 is zero
                let zero = self.builder.ins().iconst(types::I64, 0);
                let is_zero = self.builder.ins().icmp(IntCC::Equal, reg1_val, zero);

                // Select between immediate (if zero) or current reg0 value (if not zero)
                let new_val = self.builder.ins().select(is_zero, imm_val, reg0_val);
                self.builder.def_var(reg0_var, new_val);

                Ok(true) // Not a control flow instruction, needs fallthrough
            }
            Instruction::CmovNzImm(format) => {
                // Conditional move immediate if not zero: if reg1 != 0, reg0 = imm
                let reg0_var = self.registers[&format.reg0];
                let reg1_var = self.registers[&format.reg1];

                let reg1_val = self.builder.use_var(reg1_var);
                let reg0_val = self.builder.use_var(reg0_var);
                let imm_val = self.builder.ins().iconst(types::I64, format.imm0 as i64);

                // Check if reg1 is not zero
                let zero = self.builder.ins().iconst(types::I64, 0);
                let not_zero = self.builder.ins().icmp(IntCC::NotEqual, reg1_val, zero);

                // Select between immediate (if not zero) or current reg0 value (if zero)
                let new_val = self.builder.ins().select(not_zero, imm_val, reg0_val);
                self.builder.def_var(reg0_var, new_val);

                Ok(true) // Not a control flow instruction, needs fallthrough
            }
            _ => {
                // For non-control-flow instructions, use the existing visitor
                self.visit(instruction)?;
                Ok(true) // Block still needs fallthrough
            }
        }
    }
}

impl<'a, 'b> Translator<'a, 'b> {
    /// Emit a dynamic jump using Cranelift switch table for jump_indirect
    pub fn emit_dynamic_jump(&mut self, target_addr: Value, current_pc: usize, _next_pc: usize) -> Result<(), anyhow::Error> {
        // Convert target address to u32 for PVM dynamic jump protocol
        let addr_32 = self.builder.ins().ireduce(types::I32, target_addr);

        // Check for termination condition: address == u32::MAX - u16::MAX
        let termination_addr = self
            .builder
            .ins()
            .iconst(types::I32, (u32::MAX - u16::MAX as u32) as i64);
        let is_termination = self
            .builder
            .ins()
            .icmp(IntCC::Equal, addr_32, termination_addr);

        // Create blocks for error handling
        let trap_block = self.builder.create_block();
        let jump_logic_block = self.builder.create_block();

        // Branch: if termination condition, trap (end execution); else continue to jump logic
        self.builder
            .ins()
            .brif(is_termination, trap_block, &[], jump_logic_block, &[]);

        // === TRAP BLOCK (termination) ===
        self.builder.switch_to_block(trap_block);
        // Save state and return for termination
        // Get the context pointer parameter from entry block
        let entry_block = self.builder.func.layout.blocks().nth(0).unwrap();
        let context_ptr = self.builder.block_params(entry_block)[0];
        
        // Store all 13 register values back to context.registers
        for i in 0..13 {
            let reg_var = self.registers[&(i as u8)];
            let reg_value = self.builder.use_var(reg_var);
            let offset = self.builder.ins().iconst(types::I64, (i * 8) as i64);
            let addr = self.builder.ins().iadd(context_ptr, offset);
            self.builder
                .ins()
                .store(MemFlags::new(), reg_value, addr, 0);
        }
        
        // Store PC back to context.pc (offset 104) - set to termination value
        let pc_value = self.builder.ins().iconst(types::I64, i64::MAX);
        let pc_offset = self.builder.ins().iconst(types::I64, 104);
        let pc_addr = self.builder.ins().iadd(context_ptr, pc_offset);
        self.builder
            .ins()
            .store(MemFlags::new(), pc_value, pc_addr, 0);
        
        self.builder.ins().return_(&[]);
        self.builder.seal_block(trap_block);

        // === JUMP LOGIC BLOCK ===
        self.builder.switch_to_block(jump_logic_block);

        // Validate jump address: must be non-zero, aligned, and within bounds
        // Check addr != 0
        let zero = self.builder.ins().iconst(types::I32, 0);
        let addr_not_zero = self.builder.ins().icmp(IntCC::NotEqual, addr_32, zero);

        // Check alignment (addr % 2 == 0)
        let one = self.builder.ins().iconst(types::I32, 1);
        let addr_and_one = self.builder.ins().band(addr_32, one);
        let is_aligned = self.builder.ins().icmp(IntCC::Equal, addr_and_one, zero);

        // Combine validation checks
        let addr_valid = self.builder.ins().band(addr_not_zero, is_aligned);

        // Create invalid jump trap block
        let invalid_jump_block = self.builder.create_block();
        let valid_jump_block = self.builder.create_block();

        self.builder
            .ins()
            .brif(addr_valid, valid_jump_block, &[], invalid_jump_block, &[]);

        // Seal jump_logic_block since it's complete
        self.builder.seal_block(jump_logic_block);

        // === INVALID JUMP BLOCK ===
        self.builder.switch_to_block(invalid_jump_block);
        // Invalid jump - trap and return
        // Get the context pointer parameter from entry block
        let entry_block = self.builder.func.layout.blocks().nth(0).unwrap();
        let context_ptr = self.builder.block_params(entry_block)[0];
        
        // Store all 13 register values back to context.registers
        for i in 0..13 {
            let reg_var = self.registers[&(i as u8)];
            let reg_value = self.builder.use_var(reg_var);
            let offset = self.builder.ins().iconst(types::I64, (i * 8) as i64);
            let addr = self.builder.ins().iadd(context_ptr, offset);
            self.builder
                .ins()
                .store(MemFlags::new(), reg_value, addr, 0);
        }
        
        // Store PC back to context.pc - set to the current instruction PC for error handling
        let pc_value = self.builder.ins().iconst(types::I64, current_pc as i64);
        let pc_offset = self.builder.ins().iconst(types::I64, 104);
        let pc_addr = self.builder.ins().iadd(context_ptr, pc_offset);
        self.builder
            .ins()
            .store(MemFlags::new(), pc_value, pc_addr, 0);
        
        self.builder.ins().return_(&[]);
        self.builder.seal_block(invalid_jump_block);

        // === VALID JUMP BLOCK ===
        self.builder.switch_to_block(valid_jump_block);

        // Create fallback block for unknown/dynamic targets
        let fallback_block = self.builder.create_block();
        
        // Create dispatch chain for known jump targets
        // Use a series of conditional branches to check each known address
        let jump_targets: Vec<_> = self.jump_table_map.iter().collect();
        
        if jump_targets.is_empty() {
            // No known targets, go to fallback
            self.builder.ins().jump(fallback_block, &[]);
            // Seal valid_jump_block since it just has a jump
            self.builder.seal_block(valid_jump_block);
        } else {
            // Create conditional chain for each known target
            let mut current_block = valid_jump_block;
            
            for (i, (&address, &pc_target)) in jump_targets.iter().enumerate() {
                // Check if we have a basic block for this PC target
                if let Some(&target_block) = self.basic_blocks.get(&pc_target) {
                    // Check if the computed address matches this known address
                    let expected_addr = self.builder.ins().iconst(types::I32, address as i64);
                    let addr_matches = self.builder.ins().icmp(IntCC::Equal, addr_32, expected_addr);
                    
                    // Determine the next block in the chain
                    let next_check_block = if i == jump_targets.len() - 1 {
                        fallback_block // Last check, use fallback for unmatched
                    } else {
                        // Create a new block for the next check
                        let next_block = self.builder.create_block();
                        next_block
                    };
                    
                    // Branch: if address matches, jump to target block; otherwise continue checking
                    self.builder.ins().brif(addr_matches, target_block, &[], next_check_block, &[]);
                    
                    // Seal the current block and move to the next
                    self.builder.seal_block(current_block);
                    
                    // Switch to the next check block for subsequent iterations
                    if i < jump_targets.len() - 1 {
                        current_block = next_check_block;
                        self.builder.switch_to_block(current_block);
                    } else {
                        // Last iteration, seal the final block if it's not fallback_block
                        if next_check_block != fallback_block {
                            self.builder.seal_block(next_check_block);
                        }
                    }
                } else {
                    // Target block doesn't exist, go to fallback
                    self.builder.ins().jump(fallback_block, &[]);
                    self.builder.seal_block(current_block);
                    break;
                }
            }
        }
        
        // Note: valid_jump_block is sealed in the dispatch logic above
        
        // === FALLBACK BLOCK (unknown jump target) ===
        self.builder.switch_to_block(fallback_block);
        
        // For truly dynamic jumps that aren't in our static jump table,
        // we need to save state and return to the runtime
        // Get the context pointer parameter from entry block
        let entry_block = self.builder.func.layout.blocks().nth(0).unwrap();
        let context_ptr = self.builder.block_params(entry_block)[0];
        
        // Convert address back to PC for storage
        // PC = (addr / 2) - 1, then look up in jump table
        let two = self.builder.ins().iconst(types::I32, 2);
        let addr_div_2 = self.builder.ins().udiv(addr_32, two);
        let _index = self.builder.ins().isub(addr_div_2, one);
        
        // For now, just store the computed address as the PC
        // The runtime will need to resolve this
        let pc_value = self.builder.ins().uextend(types::I64, addr_32);
        
        // Store all 13 register values back to context.registers
        for i in 0..13 {
            let reg_var = self.registers[&(i as u8)];
            let reg_value = self.builder.use_var(reg_var);
            let offset = self.builder.ins().iconst(types::I64, (i * 8) as i64);
            let addr = self.builder.ins().iadd(context_ptr, offset);
            self.builder
                .ins()
                .store(MemFlags::new(), reg_value, addr, 0);
        }
        
        // Store PC back to context.pc (offset 104)
        let pc_offset = self.builder.ins().iconst(types::I64, 104);
        let pc_addr = self.builder.ins().iadd(context_ptr, pc_offset);
        self.builder
            .ins()
            .store(MemFlags::new(), pc_value, pc_addr, 0);
        
        self.builder.ins().return_(&[]);
        self.builder.seal_block(fallback_block);

        Ok(())
    }
}
