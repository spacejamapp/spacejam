//! Control flow related interfaces

use crate::{
    constants::{context_offsets, exec_result, JUMP_ALIGNMENT_FACTOR},
    Translator,
};
use anyhow::Result;
use cranelift::prelude::*;

impl<'b> Translator<'b> {
    /// Generic helper for unified branch generation
    pub fn generate_unified_branch(
        &mut self,
        condition: Value,
        target_pc: u64,
        next_pc: u64,
    ) -> Result<()> {
        if let (Some(&target_block), Some(&next_block)) =
            (self.blocks.get(&target_pc), self.blocks.get(&next_pc))
        {
            self.builder
                .ins()
                .brif(condition, target_block, &[], next_block, &[]);
        } else if let Some(&target_block) = self.blocks.get(&target_pc) {
            let cont_block = self.builder.create_block();
            self.builder
                .ins()
                .brif(condition, target_block, &[], cont_block, &[]);
            self.builder.switch_to_block(cont_block);
            self.return_continue()?;
        } else if let Some(&next_block) = self.blocks.get(&next_pc) {
            let jump_block = self.builder.create_block();
            self.builder
                .ins()
                .brif(condition, jump_block, &[], next_block, &[]);
            self.builder.switch_to_block(jump_block);
            self.return_with_jump_result(target_pc)?;
        } else {
            let jump_block = self.builder.create_block();
            let cont_block = self.builder.create_block();
            self.builder
                .ins()
                .brif(condition, jump_block, &[], cont_block, &[]);

            self.builder.switch_to_block(jump_block);
            self.return_with_jump_result(target_pc)?;

            self.builder.switch_to_block(cont_block);
            self.return_continue()?;
        }
        Ok(())
    }

    /// Return with continue result (used by empty blocks)
    pub fn return_continue(&mut self) -> Result<()> {
        // For empty blocks, use PC 0 as we don't have a specific PC
        self.return_continue_with_pc(0)
    }

    /// Return with continue result and specific PC
    pub fn return_continue_with_pc(&mut self, pc: u64) -> Result<()> {
        let ctx_ptr = self
            .get_context_ptr()
            .expect("Context pointer not initialized");

        // Save all registers back to context before returning
        self.save_registers()?;

        // Save the PC
        let pc_offset = self
            .builder
            .ins()
            .iconst(types::I64, context_offsets::PC_OFFSET as i64);
        let pc_addr = self.builder.ins().iadd(ctx_ptr, pc_offset);
        let pc_val = self.builder.ins().iconst(types::I64, pc as i64);
        self.builder
            .ins()
            .store(MemFlags::new(), pc_val, pc_addr, 0);

        let result_offset = self
            .builder
            .ins()
            .iconst(types::I64, context_offsets::RESULT_OFFSET as i64);
        let result_addr = self.builder.ins().iadd(ctx_ptr, result_offset);
        let continue_discriminant = self
            .builder
            .ins()
            .iconst(types::I64, exec_result::CONTINUE as i64);
        self.builder
            .ins()
            .store(MemFlags::new(), continue_discriminant, result_addr, 0);
        self.builder.ins().return_(&[]);
        Ok(())
    }

    /// Return with trap result
    pub fn return_trap(&mut self) -> Result<()> {
        let ctx_ptr = self
            .get_context_ptr()
            .expect("Context pointer not initialized");

        // Save all registers back to context before returning
        self.save_registers()?;

        let result_offset = self
            .builder
            .ins()
            .iconst(types::I64, context_offsets::RESULT_OFFSET as i64);
        let result_addr = self.builder.ins().iadd(ctx_ptr, result_offset);
        let trap_discriminant = self
            .builder
            .ins()
            .iconst(types::I64, exec_result::TRAP as i64);
        self.builder
            .ins()
            .store(MemFlags::new(), trap_discriminant, result_addr, 0);
        self.builder.ins().return_(&[]);
        Ok(())
    }

    /// Return with trap result and set PC to the trap instruction location
    pub fn return_trap_with_pc(&mut self, trap_pc: usize) -> Result<()> {
        let ctx_ptr = self
            .get_context_ptr()
            .expect("Context pointer not initialized");

        // Save all registers back to context before returning
        self.save_registers()?;

        // Set PC to the trap instruction location
        let pc_offset = self
            .builder
            .ins()
            .iconst(types::I64, context_offsets::PC_OFFSET as i64);
        let pc_addr = self.builder.ins().iadd(ctx_ptr, pc_offset);
        let pc_val = self.builder.ins().iconst(types::I64, trap_pc as i64);
        self.builder
            .ins()
            .store(MemFlags::new(), pc_val, pc_addr, 0);

        let result_offset = self
            .builder
            .ins()
            .iconst(types::I64, context_offsets::RESULT_OFFSET as i64);
        let result_addr = self.builder.ins().iadd(ctx_ptr, result_offset);
        let trap_discriminant = self
            .builder
            .ins()
            .iconst(types::I64, exec_result::TRAP as i64);
        self.builder
            .ins()
            .store(MemFlags::new(), trap_discriminant, result_addr, 0);
        self.builder.ins().return_(&[]);
        Ok(())
    }

    /// Return with jump result
    pub fn return_with_jump_result(&mut self, target_pc: u64) -> Result<()> {
        let ctx_ptr = self
            .get_context_ptr()
            .expect("Context pointer not initialized");

        // Save all registers back to context before returning
        self.save_registers()?;

        let result_offset = self
            .builder
            .ins()
            .iconst(types::I64, context_offsets::RESULT_OFFSET as i64);
        let result_addr = self.builder.ins().iadd(ctx_ptr, result_offset);

        let jump_discriminant = self
            .builder
            .ins()
            .iconst(types::I64, exec_result::JUMP as i64);
        self.builder
            .ins()
            .store(MemFlags::new(), jump_discriminant, result_addr, 0);

        let data_offset = self.builder.ins().iconst(types::I64, 8);
        let data_addr = self.builder.ins().iadd(result_addr, data_offset);
        let target_val = self.builder.ins().iconst(types::I64, target_pc as i64);
        self.builder
            .ins()
            .store(MemFlags::new(), target_val, data_addr, 0);

        self.builder.ins().return_(&[]);
        Ok(())
    }

    /// Set trap result in the context
    pub fn set_trap_result(&mut self, ctx_ptr: Value) -> Result<()> {
        let result_offset = self
            .builder
            .ins()
            .iconst(types::I64, context_offsets::RESULT_OFFSET as i64);
        let result_addr = self.builder.ins().iadd(ctx_ptr, result_offset);

        let trap_discriminant = self
            .builder
            .ins()
            .iconst(types::I64, exec_result::TRAP as i64);
        self.builder
            .ins()
            .store(MemFlags::new(), trap_discriminant, result_addr, 0);

        // For page faults specifically, set PC to block start
        // But for other traps, PC is already set correctly by the instruction visitor
        // TODO: Only set PC for page fault traps, not all traps

        Ok(())
    }

    /// Handle block termination using native Cranelift control flow
    pub fn handle_unified_block_termination(&mut self, block: &crate::Block) -> Result<()> {
        if let Some(last_instruction) = block.instructions.last() {
            let pc = last_instruction.range.start;

            match &last_instruction.value {
                // Handle jumps with native Cranelift control flow
                parser::Instruction::Jump(format) => {
                    let target_pc = (pc as i64 + format.off0 as i64) as u64;
                    if let Some(&target_block) = self.blocks.get(&target_pc) {
                        tracing::info!("Jumping to target block: {}", target_pc);
                        self.builder.ins().jump(target_block, &[]);
                    } else {
                        tracing::error!("Jump to unknown target: {}", target_pc);
                        // Jump to unknown target - return with jump result
                        self.return_with_jump_result(target_pc)?;
                    }
                }
                // Handle LoadImmJump with native Cranelift control flow
                parser::Instruction::LoadImmJump(format) => {
                    let target_pc = (pc as i64 + format.off0 as i64) as u64;
                    if let Some(&target_block) = self.blocks.get(&target_pc) {
                        self.builder.ins().jump(target_block, &[]);
                    } else {
                        // Jump to unknown target - return with jump result
                        self.return_with_jump_result(target_pc)?;
                    }
                }

                // Handle branches with native Cranelift control flow
                parser::Instruction::BranchEq(format) => {
                    self.handle_branch_eq_unified(&format, pc, last_instruction.range.end)?;
                }
                parser::Instruction::BranchNe(format) => {
                    self.handle_branch_ne_unified(&format, pc, last_instruction.range.end)?;
                }
                parser::Instruction::BranchEqImm(format) => {
                    self.handle_branch_eq_imm_unified(&format, pc, last_instruction.range.end)?;
                }
                parser::Instruction::BranchNeImm(format) => {
                    self.handle_branch_ne_imm_unified(&format, pc, last_instruction.range.end)?;
                }
                // Add other branch variants as needed
                parser::Instruction::BranchLtU(format) => {
                    self.handle_branch_lt_u_unified(&format, pc, last_instruction.range.end)?;
                }
                parser::Instruction::BranchLtS(format) => {
                    self.handle_branch_lt_s_unified(&format, pc, last_instruction.range.end)?;
                }
                parser::Instruction::BranchGeU(format) => {
                    self.handle_branch_ge_u_unified(&format, pc, last_instruction.range.end)?;
                }
                parser::Instruction::BranchGeS(format) => {
                    self.handle_branch_ge_s_unified(&format, pc, last_instruction.range.end)?;
                }
                // Handle immediate variants
                parser::Instruction::BranchLtUImm(format) => {
                    self.handle_branch_lt_u_imm_unified(&format, pc, last_instruction.range.end)?;
                }
                parser::Instruction::BranchLtSImm(format) => {
                    self.handle_branch_lt_s_imm_unified(&format, pc, last_instruction.range.end)?;
                }
                parser::Instruction::BranchGeUImm(format) => {
                    self.handle_branch_ge_u_imm_unified(&format, pc, last_instruction.range.end)?;
                }
                parser::Instruction::BranchGeSImm(format) => {
                    self.handle_branch_ge_s_imm_unified(&format, pc, last_instruction.range.end)?;
                }
                parser::Instruction::BranchGtUImm(format) => {
                    self.handle_branch_gt_u_imm_unified(&format, pc, last_instruction.range.end)?;
                }
                parser::Instruction::BranchGtSImm(format) => {
                    self.handle_branch_gt_s_imm_unified(&format, pc, last_instruction.range.end)?;
                }
                parser::Instruction::BranchLeUImm(format) => {
                    self.handle_branch_le_u_imm_unified(&format, pc, last_instruction.range.end)?;
                }
                parser::Instruction::BranchLeSImm(format) => {
                    self.handle_branch_le_s_imm_unified(&format, pc, last_instruction.range.end)?;
                }

                // Handle indirect jumps with runtime dispatch
                parser::Instruction::JumpInd(_) | parser::Instruction::LoadImmJumpInd(_) => {
                    // In unified mode, generate a runtime switch to dispatch to the target block
                    self.handle_indirect_jump_unified(pc)?;
                }

                // Handle traps and halts
                parser::Instruction::Trap => {
                    self.return_trap_with_pc(pc)?;
                }

                _ => {
                    // Non-terminating instruction - fall through to next block
                    let next_pc = last_instruction.range.end as u64;
                    if let Some(&next_block) = self.blocks.get(&next_pc) {
                        self.builder.ins().jump(next_block, &[]);
                    } else {
                        // No next block - program ends at the end of this instruction
                        self.return_continue_with_pc(next_pc)?;
                    }
                }
            }
        } else {
            // Empty block - continue
            self.return_continue()?;
        }

        Ok(())
    }

    /// Handle indirect jump in unified mode - generate runtime dispatch with proper validation
    pub fn handle_indirect_jump_unified(&mut self, instruction_pc: usize) -> Result<()> {
        let ctx_ptr = self
            .get_context_ptr()
            .expect("Context pointer not initialized");

        // Read the target address that was computed and stored by the visitor
        let result_offset = self
            .builder
            .ins()
            .iconst(types::I64, context_offsets::RESULT_OFFSET as i64);
        let result_addr = self.builder.ins().iadd(ctx_ptr, result_offset);
        let data_offset = self.builder.ins().iconst(types::I64, 8);
        let data_addr = self.builder.ins().iadd(result_addr, data_offset);
        let target_addr = self
            .builder
            .ins()
            .load(types::I64, MemFlags::new(), data_addr, 0);

        // Implement the same validation logic as the interpreter:
        // 1. address == 0 (null address)
        // 2. address > table.len() * JUMP_ALIGNMENT_FACTOR (beyond table bounds)
        // 3. address % 2 != 0 (not aligned to 2-byte boundary)

        let _current_block = self.builder.current_block().unwrap();
        let valid_jump_block = self.builder.create_block();
        let trap_block = self.builder.create_block();

        // Check 1: address == 0
        let zero = self.builder.ins().iconst(types::I64, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, target_addr, zero);

        // Check 2: address > table.len() * JUMP_ALIGNMENT_FACTOR
        let table_len = self.jump_table.len() as u32;
        let jump_alignment_factor = JUMP_ALIGNMENT_FACTOR;
        let max_address = table_len * jump_alignment_factor;
        let max_addr_val = self.builder.ins().iconst(types::I64, max_address as i64);
        let exceeds_bounds =
            self.builder
                .ins()
                .icmp(IntCC::UnsignedGreaterThan, target_addr, max_addr_val);

        // Check 3: address % 2 != 0 (misaligned)
        let two = self.builder.ins().iconst(types::I64, 2);
        let remainder = self.builder.ins().urem(target_addr, two);
        let is_misaligned = self.builder.ins().icmp(IntCC::NotEqual, remainder, zero);

        // Combine all invalid conditions with OR
        let invalid1 = self.builder.ins().bor(is_zero, exceeds_bounds);
        let invalid_jump = self.builder.ins().bor(invalid1, is_misaligned);

        // Branch: if invalid, trap; otherwise continue to valid jump handling
        self.builder
            .ins()
            .brif(invalid_jump, trap_block, &[], valid_jump_block, &[]);

        // Valid jump block: calculate index and dispatch
        self.builder.switch_to_block(valid_jump_block);

        // Calculate jump table index: (address / 2) - 1 (following interpreter logic)
        let addr_div_2 = self.builder.ins().udiv(target_addr, two);
        let one = self.builder.ins().iconst(types::I64, 1);
        let jump_index = self.builder.ins().isub(addr_div_2, one);

        // Create switch to dispatch to correct block based on jump table index
        let mut switch = cranelift::frontend::Switch::new();

        // Add all jump table entries as switch cases
        for (i, &jump_pc) in self.jump_table.iter().enumerate() {
            if let Some(&cranelift_block) = self.blocks.get(&jump_pc) {
                switch.set_entry(i as u128, cranelift_block);
            }
        }

        // Emit the switch (default case goes to trap for out-of-bounds indices)
        switch.emit(&mut self.builder, jump_index, trap_block);

        // Trap block: invalid jump target
        self.builder.switch_to_block(trap_block);
        self.return_trap_with_pc(instruction_pc)?;

        // Seal all created blocks
        self.builder.seal_block(valid_jump_block);
        self.builder.seal_block(trap_block);

        Ok(())
    }

    /// Handle BranchEq instruction in unified mode with native Cranelift control flow
    pub fn handle_branch_eq_unified(
        &mut self,
        format: &parser::format::RRO,
        pc: usize,
        next_pc: usize,
    ) -> Result<()> {
        let parser::format::RRO { reg0, reg1, off0 } = *format;

        // Compare registers
        let reg0_val = self.builder.use_var(self.registers[&reg0]);
        let reg1_val = self.builder.use_var(self.registers[&reg1]);
        let condition = self.builder.ins().icmp(IntCC::Equal, reg0_val, reg1_val);

        // Calculate target addresses
        let target_pc = (pc as i64 + off0 as i64) as u64;

        self.generate_unified_branch(condition, target_pc, next_pc as u64)
    }

    /// Handle BranchNe instruction in unified mode
    pub fn handle_branch_ne_unified(
        &mut self,
        format: &parser::format::RRO,
        pc: usize,
        next_pc: usize,
    ) -> Result<()> {
        let parser::format::RRO { reg0, reg1, off0 } = *format;
        let reg0_val = self.builder.use_var(self.registers[&reg0]);
        let reg1_val = self.builder.use_var(self.registers[&reg1]);
        let condition = self.builder.ins().icmp(IntCC::NotEqual, reg0_val, reg1_val);

        let target_pc = (pc as i64 + off0 as i64) as u64;

        self.generate_unified_branch(condition, target_pc, next_pc as u64)
    }

    /// Handle BranchEqImm instruction in unified mode
    pub fn handle_branch_eq_imm_unified(
        &mut self,
        format: &parser::format::RIO,
        pc: usize,
        next_pc: usize,
    ) -> Result<()> {
        let parser::format::RIO { reg0, imm0, off0 } = *format;
        let reg_val = self.builder.use_var(self.registers[&reg0]);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = self.builder.ins().icmp(IntCC::Equal, reg_val, imm_val);

        let target_pc = (pc as i64 + off0 as i64) as u64;

        self.generate_unified_branch(condition, target_pc, next_pc as u64)
    }

    /// Handle BranchNeImm instruction in unified mode
    pub fn handle_branch_ne_imm_unified(
        &mut self,
        format: &parser::format::RIO,
        pc: usize,
        next_pc: usize,
    ) -> Result<()> {
        let parser::format::RIO { reg0, imm0, off0 } = *format;
        let reg_val = self.builder.use_var(self.registers[&reg0]);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = self.builder.ins().icmp(IntCC::NotEqual, reg_val, imm_val);

        let target_pc = (pc as i64 + off0 as i64) as u64;

        self.generate_unified_branch(condition, target_pc, next_pc as u64)
    }

    // Implementations for all branch types
    pub fn handle_branch_lt_u_unified(
        &mut self,
        format: &parser::format::RRO,
        pc: usize,
        next_pc: usize,
    ) -> Result<()> {
        let parser::format::RRO { reg0, reg1, off0 } = *format;
        let reg0_val = self.builder.use_var(self.registers[&reg0]);
        let reg1_val = self.builder.use_var(self.registers[&reg1]);
        let condition = self
            .builder
            .ins()
            .icmp(IntCC::UnsignedLessThan, reg0_val, reg1_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(condition, target_pc, next_pc as u64)
    }

    pub fn handle_branch_lt_s_unified(
        &mut self,
        format: &parser::format::RRO,
        pc: usize,
        next_pc: usize,
    ) -> Result<()> {
        let parser::format::RRO { reg0, reg1, off0 } = *format;
        let reg0_val = self.builder.use_var(self.registers[&reg0]);
        let reg1_val = self.builder.use_var(self.registers[&reg1]);
        let condition = self
            .builder
            .ins()
            .icmp(IntCC::SignedLessThan, reg0_val, reg1_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(condition, target_pc, next_pc as u64)
    }

    pub fn handle_branch_ge_u_unified(
        &mut self,
        format: &parser::format::RRO,
        pc: usize,
        next_pc: usize,
    ) -> Result<()> {
        let parser::format::RRO { reg0, reg1, off0 } = *format;
        let reg0_val = self.builder.use_var(self.registers[&reg0]);
        let reg1_val = self.builder.use_var(self.registers[&reg1]);
        let condition =
            self.builder
                .ins()
                .icmp(IntCC::UnsignedGreaterThanOrEqual, reg0_val, reg1_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(condition, target_pc, next_pc as u64)
    }

    pub fn handle_branch_ge_s_unified(
        &mut self,
        format: &parser::format::RRO,
        pc: usize,
        next_pc: usize,
    ) -> Result<()> {
        let parser::format::RRO { reg0, reg1, off0 } = *format;
        let reg0_val = self.builder.use_var(self.registers[&reg0]);
        let reg1_val = self.builder.use_var(self.registers[&reg1]);
        let condition =
            self.builder
                .ins()
                .icmp(IntCC::SignedGreaterThanOrEqual, reg0_val, reg1_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(condition, target_pc, next_pc as u64)
    }

    pub fn handle_branch_lt_u_imm_unified(
        &mut self,
        format: &parser::format::RIO,
        pc: usize,
        next_pc: usize,
    ) -> Result<()> {
        let parser::format::RIO { reg0, imm0, off0 } = *format;
        let reg_val = self.builder.use_var(self.registers[&reg0]);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = self
            .builder
            .ins()
            .icmp(IntCC::UnsignedLessThan, reg_val, imm_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(condition, target_pc, next_pc as u64)
    }

    pub fn handle_branch_lt_s_imm_unified(
        &mut self,
        format: &parser::format::RIO,
        pc: usize,
        next_pc: usize,
    ) -> Result<()> {
        let parser::format::RIO { reg0, imm0, off0 } = *format;
        let reg_val = self.builder.use_var(self.registers[&reg0]);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = self
            .builder
            .ins()
            .icmp(IntCC::SignedLessThan, reg_val, imm_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(condition, target_pc, next_pc as u64)
    }

    pub fn handle_branch_ge_u_imm_unified(
        &mut self,
        format: &parser::format::RIO,
        pc: usize,
        next_pc: usize,
    ) -> Result<()> {
        let parser::format::RIO { reg0, imm0, off0 } = *format;
        let reg_val = self.builder.use_var(self.registers[&reg0]);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let condition =
            self.builder
                .ins()
                .icmp(IntCC::UnsignedGreaterThanOrEqual, reg_val, imm_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(condition, target_pc, next_pc as u64)
    }

    pub fn handle_branch_ge_s_imm_unified(
        &mut self,
        format: &parser::format::RIO,
        pc: usize,
        next_pc: usize,
    ) -> Result<()> {
        let parser::format::RIO { reg0, imm0, off0 } = *format;
        let reg_val = self.builder.use_var(self.registers[&reg0]);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = self
            .builder
            .ins()
            .icmp(IntCC::SignedGreaterThanOrEqual, reg_val, imm_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(condition, target_pc, next_pc as u64)
    }

    pub fn handle_branch_gt_u_imm_unified(
        &mut self,
        format: &parser::format::RIO,
        pc: usize,
        next_pc: usize,
    ) -> Result<()> {
        let parser::format::RIO { reg0, imm0, off0 } = *format;
        let reg_val = self.builder.use_var(self.registers[&reg0]);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = self
            .builder
            .ins()
            .icmp(IntCC::UnsignedGreaterThan, reg_val, imm_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(condition, target_pc, next_pc as u64)
    }

    pub fn handle_branch_gt_s_imm_unified(
        &mut self,
        format: &parser::format::RIO,
        pc: usize,
        next_pc: usize,
    ) -> Result<()> {
        let parser::format::RIO { reg0, imm0, off0 } = *format;
        let reg_val = self.builder.use_var(self.registers[&reg0]);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = self
            .builder
            .ins()
            .icmp(IntCC::SignedGreaterThan, reg_val, imm_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(condition, target_pc, next_pc as u64)
    }

    pub fn handle_branch_le_u_imm_unified(
        &mut self,
        format: &parser::format::RIO,
        pc: usize,
        next_pc: usize,
    ) -> Result<()> {
        let parser::format::RIO { reg0, imm0, off0 } = *format;
        let reg_val = self.builder.use_var(self.registers[&reg0]);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = self
            .builder
            .ins()
            .icmp(IntCC::UnsignedLessThanOrEqual, reg_val, imm_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(condition, target_pc, next_pc as u64)
    }

    pub fn handle_branch_le_s_imm_unified(
        &mut self,
        format: &parser::format::RIO,
        pc: usize,
        next_pc: usize,
    ) -> Result<()> {
        let parser::format::RIO { reg0, imm0, off0 } = *format;
        let reg_val = self.builder.use_var(self.registers[&reg0]);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = self
            .builder
            .ins()
            .icmp(IntCC::SignedLessThanOrEqual, reg_val, imm_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(condition, target_pc, next_pc as u64)
    }
}
