//! Context of the translator

use crate::{Registers, Translator};
use anyhow::Result;
use cranelift::prelude::InstBuilder;
use cranelift_frontend::FunctionBuilder;
use parser::{format, reader::Offset, Instruction};
use pvm::Visitor;
use std::ops::{Deref, DerefMut, Range};

/// Context of the translator
pub struct Context<'b> {
    /// The registers of the context
    pub pool: Registers,

    /// The builder of the context
    pub builder: FunctionBuilder<'b>,
}

impl Context<'_> {
    /// Burn gas for an instruction
    pub fn burn_gas(&mut self, instr: &Offset<Instruction>) -> Result<()> {
        self.dispatch(instr.value, &instr.range)
    }

    /// Burn gas for an instruction
    pub fn burn_gas_imm(&mut self, amount: i64) -> Result<()> {
        let mut gas = self.builder.use_var(self.pool.gas);
        gas = self.builder.ins().iadd_imm(gas, amount);
        self.builder.def_var(self.pool.gas, gas);
        Ok(())
    }
}

impl<'b> Deref for Translator<'b> {
    type Target = Context<'b>;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl<'b> DerefMut for Translator<'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}

impl Visitor for Context<'_> {
    type Error = anyhow::Error;
    type Output = ();

    fn visit_default(&mut self) -> Result<Self::Output, Self::Error> {
        let mut gas = self.builder.use_var(self.pool.gas);
        gas = self.builder.ins().iadd_imm(gas, -1);
        self.builder.def_var(self.pool.gas, gas);
        Ok(())
    }

    fn visit_ecalli(
        &mut self,
        format: format::I,
        _range: &Range<usize>,
    ) -> Result<Self::Output, Self::Error> {
        let format::I { imm0: call } = format;
        let mut gas = self.builder.use_var(self.pool.gas);
        let gas = match call {
            20 => {
                let reg9 = self.builder.use_var(self.pool.registers[9]);
                gas = self.builder.ins().iadd_imm(gas, -11);
                self.builder.ins().isub(gas, reg9)
            }
            100 => self.builder.ins().iadd_imm(gas, -1),
            _ => self.builder.ins().iadd_imm(gas, -11),
        };

        self.builder.def_var(self.pool.gas, gas);
        Ok(())
    }
}
