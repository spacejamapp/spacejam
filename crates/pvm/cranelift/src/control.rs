//! Control flow related interfaces

use crate::{context::offsets, exit::Exit, Translator};
use anyhow::Result;
use cranelift::prelude::*;

const HALT_TARGET: u64 = (u32::MAX - u16::MAX as u32) as u64;

impl Translator<'_> {
    /// Check if the pc needs to sync
    pub fn need_sync(&self, pc: &u64) -> bool {
        self.jump.contains(pc) || self.testing
    }

    /// burn gas (subtract from the gas counter using SSA)
    ///
    /// TODO: handle OOG
    pub fn burn_gas(&mut self, amount: i64) {
        // Use SSA subtraction instead of memory load/store
        self.pool.gas = self.builder.ins().iadd_imm(self.pool.gas, amount);
    }

    /// get pc from the context
    pub fn pc(&mut self) -> Value {
        let offset = self
            .builder
            .ins()
            .iconst(types::I64, offsets::PC_OFFSET as i64);
        let addr = self.builder.ins().iadd(self.pool.ctx, offset);
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
        let addr = self.builder.ins().iadd(self.pool.ctx, offset);
        let pc_val = self.builder.ins().iconst(types::I64, pc as i64);
        self.builder
            .ins()
            .store(MemFlags::trusted(), pc_val, addr, 0);
    }

    /// generate branch instruction
    pub fn branch(&mut self, condition: Value, target_pc: u64, next_pc: u64) -> Result<()> {
        let target_block = self.blocks[&target_pc];
        let next_block = self.blocks[&next_pc];

        // Check if blocks expect parameters or load from memory
        let target_needs_sync = self.need_sync(&target_pc);
        let next_needs_sync = self.need_sync(&next_pc);
        let empty_args: Vec<cranelift_codegen::ir::BlockArg> = vec![];
        let args = self.block_args();
        if target_needs_sync || next_needs_sync {
            self.sync_params();
        }

        // switch the arguments based on the needs
        let (target_args, next_args) = {
            let target_args = if target_needs_sync {
                &empty_args[..]
            } else {
                &args[..]
            };
            let next_args = if next_needs_sync {
                &empty_args[..]
            } else {
                &args[..]
            };
            (target_args, next_args)
        };

        self.builder
            .ins()
            .brif(condition, target_block, target_args, next_block, next_args);
        Ok(())
    }

    /// Return with trap result and set PC to the trap instruction location
    pub fn return_(&mut self, exit: Exit) {
        self.sync_params();
        let res = exit.value(&mut self.builder);
        self.builder.ins().return_(&[res]);
    }

    /// Handle indirect jump - generate runtime dispatch with proper validation
    pub fn djump(&mut self, target: Value) -> Result<()> {
        let halt_block = self.builder.create_block();
        let check_valid = self.builder.create_block();
        let halt = self.builder.ins().iconst(types::I64, HALT_TARGET as i64);
        let is_halt = self.builder.ins().icmp(IntCC::Equal, target, halt);
        self.builder
            .ins()
            .brif(is_halt, halt_block, &[], check_valid, &[]);

        // Halt block: return halt result
        self.builder.switch_to_block(halt_block);
        self.return_(Exit::Halt);

        // Jump target validation:
        // 1. address == 0 (null address)
        // 2. address > table.len() * JUMP_ALIGNMENT_FACTOR (beyond table bounds)
        // 3. address % 2 != 0 (not aligned to 2-byte boundary)
        self.builder.switch_to_block(check_valid);
        let valid = self.builder.create_block();
        let trap = self.builder.create_block();
        let two = self.builder.ins().iconst(types::I64, 2);
        {
            // Check 1: address == 0
            let zero = self.builder.ins().iconst(types::I64, 0);
            let is_zero = self.builder.ins().icmp(IntCC::Equal, target, zero);

            // Check 2: address > table.len() * JUMP_ALIGNMENT_FACTOR
            let table_len = self.jump.len() as u32;
            let max_address = table_len * pvm::JUMP_ALIGNMENT_FACTOR;
            let max_addr_val = self.builder.ins().iconst(types::I64, max_address as i64);
            let exceeds_bounds =
                self.builder
                    .ins()
                    .icmp(IntCC::UnsignedGreaterThan, target, max_addr_val);

            // Check 3: address % 2 != 0 (misaligned)
            let remainder = self.builder.ins().urem(target, two);
            let is_misaligned = self.builder.ins().icmp(IntCC::NotEqual, remainder, zero);

            // Combine all invalid conditions with OR
            let invalid = self.builder.ins().bor(is_zero, exceeds_bounds);
            let invalid_jump = self.builder.ins().bor(invalid, is_misaligned);
            self.builder.ins().brif(invalid_jump, trap, &[], valid, &[]);
        }

        // Valid jump block: calculate index and dispatch
        self.builder.switch_to_block(valid);
        {
            self.sync_params();

            // Calculate jump table index: (address / 2) - 1
            let addr_div_2 = self.builder.ins().udiv(target, two);
            let one = self.builder.ins().iconst(types::I64, 1);
            let jump_index = self.builder.ins().isub(addr_div_2, one);
            let jump_index = self.builder.ins().ireduce(types::I32, jump_index);
            self.builder.ins().br_table(jump_index, self.rt_jump_table);
        }

        // Trap block: invalid jump target
        self.builder.switch_to_block(trap);
        self.return_(Exit::InvalidJumpTarget);

        // Seal all created blocks
        self.builder.seal_block(halt_block);
        self.builder.seal_block(check_valid);
        self.builder.seal_block(valid);
        self.builder.seal_block(trap);
        Ok(())
    }
}
