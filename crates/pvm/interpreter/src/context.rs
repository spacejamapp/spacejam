//! Context of the interpreter

use pvm::Argument;
use std::{cell::RefCell, rc::Rc};

/// Context of the interpreter
#[derive(Default)]
pub struct Context {
    /// The gas of the interpreter.
    pub gas: i64,

    /// The memory of the interpreter.
    pub memory: Rc<RefCell<parser::Memory>>,

    /// The registers of the interpreter.
    pub registers: [u64; 13],
}

impl Context {
    /// Convert the context to the PVM context.
    pub fn ctx<X: Argument>(&self, ctx: X) -> pvm::Context<X, parser::Memory> {
        pvm::Context {
            ctx,
            memory: self.memory.clone(),
            registers: self.registers,
            gas: self.gas,
        }
    }

    /// sync from the PVM context
    pub fn sync<X: Argument>(&mut self, ctx: &pvm::Context<X, parser::Memory>) {
        self.gas = ctx.gas;
        self.registers = ctx.registers;
    }
}
