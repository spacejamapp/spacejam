//! Control flow related interfaces

use crate::{context_offsets, Translator};
use anyhow::Result;
use cranelift::prelude::*;

/// Execution result discriminant values
pub mod result {
    pub const CONTINUE: u64 = 0;
    pub const HALT: u64 = 2;
    pub const TRAP: u64 = 3;
    pub const JUMP_INDIRECT: u64 = 4;
}

impl Translator<'_> {
    /// Generic helper for branch generation
    /// Assumes both target and next blocks exist (full compilation mode)
    pub fn branch(&mut self, condition: Value, target_pc: u64, next_pc: u64) -> Result<()> {
        let target_block = self.blocks[&target_pc];
        let next_block = self.blocks[&next_pc];
        self.builder
            .ins()
            .brif(condition, target_block, &[], next_block, &[]);
        Ok(())
    }

    /// Return with continue result and specific PC
    pub fn return_continue_with_pc(&mut self, pc: u64) -> Result<()> {
        let ctx_ptr = self.ctx_ptr;

        // Save all registers back to context before returning
        self.save_registers();

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
            .iconst(types::I64, result::CONTINUE as i64);
        self.builder
            .ins()
            .store(MemFlags::new(), continue_discriminant, result_addr, 0);
        self.builder.ins().return_(&[]);
        Ok(())
    }

    /// Return with trap result
    pub fn return_trap(&mut self) -> Result<()> {
        let ctx_ptr = self.ctx_ptr;

        // Save all registers back to context before returning
        self.save_registers();

        let result_offset = self
            .builder
            .ins()
            .iconst(types::I64, context_offsets::RESULT_OFFSET as i64);
        let result_addr = self.builder.ins().iadd(ctx_ptr, result_offset);
        let trap_discriminant = self.builder.ins().iconst(types::I64, result::TRAP as i64);
        self.builder
            .ins()
            .store(MemFlags::new(), trap_discriminant, result_addr, 0);
        self.builder.ins().return_(&[]);
        Ok(())
    }

    /// Return with trap result and set PC to the trap instruction location
    pub fn return_trap_with_pc(&mut self, trap_pc: usize) -> Result<()> {
        let ctx_ptr = self.ctx_ptr;

        // Save all registers back to context before returning
        self.save_registers();

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
        let trap_discriminant = self.builder.ins().iconst(types::I64, result::TRAP as i64);
        self.builder
            .ins()
            .store(MemFlags::new(), trap_discriminant, result_addr, 0);
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

        let trap_discriminant = self.builder.ins().iconst(types::I64, result::TRAP as i64);
        self.builder
            .ins()
            .store(MemFlags::new(), trap_discriminant, result_addr, 0);

        // For page faults specifically, set PC to block start
        // But for other traps, PC is already set correctly by the instruction visitor
        // TODO: Only set PC for page fault traps, not all traps

        Ok(())
    }

    /// Handle indirect jump - generate runtime dispatch with proper validation
    pub fn djump(&mut self, instruction_pc: usize) -> Result<()> {
        let ctx_ptr = self.ctx_ptr;

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

        // Same validation logic as the interpreter:
        // 1. address == 0 (null address)
        // 2. address > table.len() * JUMP_ALIGNMENT_FACTOR (beyond table bounds)
        // 3. address % 2 != 0 (not aligned to 2-byte boundary)
        let valid_jump_block = self.builder.create_block();
        let trap_block = self.builder.create_block();

        // Check 1: address == 0
        let zero = self.builder.ins().iconst(types::I64, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, target_addr, zero);

        // Check 2: address > table.len() * JUMP_ALIGNMENT_FACTOR
        let table_len = self.jump_table.len() as u32;
        let max_address = table_len * pvm::JUMP_ALIGNMENT_FACTOR;
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
            let cranelift_block = self.blocks[&jump_pc];
            switch.set_entry(i as u128, cranelift_block);
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
}
