//! Context of the interpreter

use pvm::Argument;

/// Context of the interpreter
#[derive(Default)]
pub struct Context {
    /// The gas of the interpreter.
    pub gas: i64,

    /// The memory of the interpreter.
    pub memory: parser::Memory,

    /// The registers of the interpreter.
    pub registers: [u64; 13],
}

impl Context {
    /// Convert the context to the PVM context.
    pub fn ctx<X: Argument>(&mut self, ctx: X) -> pvm::Context<'_, X, parser::Memory> {
        pvm::Context {
            ctx,
            memory: &mut self.memory,
            registers: &mut self.registers,
            gas: &mut self.gas,
        }
    }
}
