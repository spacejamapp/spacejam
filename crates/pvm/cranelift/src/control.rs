//! Control flow related interfaces

use crate::{Translator, exit::Exit};
use anyhow::Result;
use cranelift::prelude::*;
use cranelift_codegen::ir::BlockArg;

impl Translator<'_> {
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
    pub fn return_(&mut self, exit: Exit) {
        self.sync_registers();
        let exit = exit.value(&mut self.builder);
        let gas = self.context.builder.use_var(self.context.pool.gas);
        self.builder.ins().return_(&[gas, exit]);
    }

    /// Handle indirect jump - generate runtime dispatch with proper validation
    pub fn djump(&mut self, target: Value) -> Result<()> {
        let djump = self.masm.djump;
        self.builder.ins().jump(djump, &[BlockArg::Value(target)]);
        Ok(())
    }
}
