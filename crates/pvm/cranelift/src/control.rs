//! Control flow related interfaces

use crate::{context::offsets, Translator};
use anyhow::Result;
use cranelift::prelude::*;

/// Execution result discriminant values
pub mod result {
    pub const HALT: i64 = 0;
    pub const PANIC: i64 = 1;
    pub const FAULT: i64 = 2;
    pub const HOST: i64 = 3;
    pub const OOG: i64 = 4;
}

impl Translator<'_> {
    /// get result from the context
    pub fn jump(&mut self) -> Value {
        self.builder.use_var(self.jump)
    }

    /// set dynamic jump to the context
    pub fn set_jump(&mut self, target: Value) {
        self.builder.def_var(self.jump, target);
    }

    /// get gas value from the context
    pub fn gas(&mut self) -> Value {
        let offset = self
            .builder
            .ins()
            .iconst(types::I64, offsets::GAS_OFFSET as i64);
        let addr = self.builder.ins().iadd(self.ctx_ptr, offset);
        self.builder
            .ins()
            .load(types::I64, MemFlags::trusted(), addr, 0)
    }

    /// burn gas (add to the gas counter)
    pub fn burn_gas(&mut self, gas: i64) {
        let offset = self
            .builder
            .ins()
            .iconst(types::I64, offsets::GAS_OFFSET as i64);
        let addr = self.builder.ins().iadd(self.ctx_ptr, offset);
        let current_gas = self
            .builder
            .ins()
            .load(types::I64, MemFlags::trusted(), addr, 0);
        let burn_amount = self.builder.ins().iconst(types::I64, gas);
        let result = self.builder.ins().iadd(current_gas, burn_amount);
        self.builder
            .ins()
            .store(MemFlags::trusted(), result, addr, 0);
    }

    /// get pc from the context
    pub fn pc(&mut self) -> Value {
        let offset = self
            .builder
            .ins()
            .iconst(types::I64, offsets::PC_OFFSET as i64);
        let addr = self.builder.ins().iadd(self.ctx_ptr, offset);
        self.builder
            .ins()
            .load(types::I64, MemFlags::trusted(), addr, 0)
    }

    /// set pc to the context
    pub fn set_pc(&mut self, pc: u64) {
        let offset = self
            .builder
            .ins()
            .iconst(types::I64, offsets::PC_OFFSET as i64);
        let addr = self.builder.ins().iadd(self.ctx_ptr, offset);
        let pc_val = self.builder.ins().iconst(types::I64, pc as i64);
        self.builder
            .ins()
            .store(MemFlags::trusted(), pc_val, addr, 0);
    }

    /// generate branch instruction
    pub fn branch(&mut self, condition: Value, target_pc: u64, next_pc: u64) -> Result<()> {
        let target_block = self.blocks[&target_pc];
        let next_block = self.blocks[&next_pc];
        self.builder
            .ins()
            .brif(condition, target_block, &[], next_block, &[]);
        Ok(())
    }

    /// Return with trap result and set PC to the trap instruction location
    pub fn return_(&mut self, sig: i64, pc: usize) -> Result<()> {
        self.save_registers();
        self.set_pc(pc as u64);
        let res = self.builder.ins().iconst(types::I8, sig);
        self.builder.ins().return_(&[res]);
        Ok(())
    }

    /// Handle indirect jump - generate runtime dispatch with proper validation
    pub fn djump(&mut self, pc: usize) -> Result<()> {
        let target_addr = self.jump();

        // Jump target validation:
        // 1. address == 0 (null address)
        // 2. address > table.len() * JUMP_ALIGNMENT_FACTOR (beyond table bounds)
        // 3. address % 2 != 0 (not aligned to 2-byte boundary)
        let valid = self.builder.create_block();
        let trap = self.builder.create_block();
        {
            // Check 1: address == 0
            let zero = self.builder.ins().iconst(types::I64, 0);
            let is_zero = self.builder.ins().icmp(IntCC::Equal, target_addr, zero);

            // Check 2: address > table.len() * JUMP_ALIGNMENT_FACTOR
            let table_len = self.blocks.len() as u32;
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
            self.builder.ins().brif(invalid_jump, trap, &[], valid, &[]);
        }

        // Valid jump block: calculate index and dispatch
        //
        // FIXME: do we have to generate a switch here?
        self.builder.switch_to_block(valid);
        {
            let mut switch = cranelift::frontend::Switch::new();
            for (i, block) in self.blocks.iter() {
                switch.set_entry(*i as u128, *block);
            }
            switch.emit(&mut self.builder, target_addr, trap);
        }

        // Trap block: invalid jump target
        self.builder.switch_to_block(trap);
        self.return_(result::PANIC, pc)?;

        // Seal all created blocks
        self.builder.seal_block(valid);
        self.builder.seal_block(trap);
        Ok(())
    }
}
