//! Context of the translator

use crate::{Exit, Pool, Translator};
use anyhow::Result;
use cranelift::prelude::{FunctionBuilder, InstBuilder, IntCC, Value};
use parser::{Instruction, format, reader::Offset};
use pvm::Visitor;
use std::ops::{Deref, DerefMut, Range};

/// Context of the translator
pub struct Context<'b> {
    /// The registers of the context
    pub pool: Pool,

    /// The builder of the context
    pub builder: FunctionBuilder<'b>,
}

impl Context<'_> {
    /// Burn gas for an instruction
    pub fn burn_gas(&mut self, instr: &Offset<Instruction>) -> Result<()> {
        let to_burn = self.dispatch(instr.value, &instr.range)?;
        let mut gas = self.builder.use_var(self.pool.gas);

        // if out of gas
        let oog = match to_burn {
            Gas::Imm(amount) => self
                .builder
                .ins()
                .icmp_imm(IntCC::SignedLessThan, gas, amount),
            Gas::Value(amount) => self.builder.ins().icmp(IntCC::SignedLessThan, gas, amount),
        };

        gas = match to_burn {
            Gas::Imm(amount) => self.builder.ins().iadd_imm(gas, -amount),
            Gas::Value(amount) => self.builder.ins().isub(gas, amount),
        };
        self.builder.def_var(self.pool.gas, gas);

        // returns OOG if out of gas
        let then_block = self.builder.create_block();
        let else_block = self.builder.create_block();
        self.builder
            .ins()
            .brif(oog, then_block, &[], else_block, &[]);

        self.builder.switch_to_block(then_block);
        let exit = Exit::OOG.value(&mut self.builder);
        self.builder.ins().return_(&[gas, exit]);

        // do nothing if not out of gas
        self.builder.switch_to_block(else_block);
        Ok(())
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
    type Output = Gas;

    fn visit_default(&mut self) -> Result<Self::Output, Self::Error> {
        Ok(Gas::Imm(1))
    }

    fn visit_ecalli(
        &mut self,
        format: format::I,
        _range: &Range<usize>,
    ) -> Result<Self::Output, Self::Error> {
        let format::I { imm0: call } = format;
        Ok(match call {
            20 => {
                let reg9 = self.builder.use_var(self.pool.registers[9]);
                Gas::Value(self.builder.ins().iadd_imm(reg9, 11))
            }
            100 => Gas::Imm(1),
            _ => Gas::Imm(11),
        })
    }
}

/// Gas spent
pub enum Gas {
    Imm(i64),
    Value(Value),
}
