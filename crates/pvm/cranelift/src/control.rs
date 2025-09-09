//! Control flow related interfaces

use crate::{exit::Exit, Translator};
use anyhow::Result;
use cranelift::prelude::*;
use cranelift_codegen::ir::BlockArg;

impl Translator<'_> {
    /// burn gas (subtract from the gas counter using SSA)
    pub fn burn_gas(&mut self, amount: i64) {
        let mut gas = self.builder.use_var(self.pool.gas);
        gas = self.builder.ins().iadd_imm(gas, amount);
        self.builder.def_var(self.pool.gas, gas);
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
    pub fn return_(&mut self, exit: Exit) {
        self.sync_registers();
        let exit = exit.value(&mut self.builder);
        let gas = self.builder.use_var(self.pool.gas);
        self.builder.ins().return_(&[gas, exit]);
    }

    /// Handle indirect jump - generate runtime dispatch with proper validation
    pub fn djump(&mut self, target: Value) -> Result<()> {
        self.builder
            .ins()
            .jump(self.masm.djump, &[BlockArg::Value(target)]);
        Ok(())
    }
}
