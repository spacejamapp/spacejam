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
    pub fn ctx<'ctx, X: Argument>(
        &'ctx mut self,
        ctx: &'ctx mut X,
    ) -> pvm::Context<'ctx, X, &'ctx mut parser::Memory> {
        pvm::Context {
            ctx,
            memory: &mut self.memory,
            pc: 0,
            registers: self.registers,
            gas: self.gas,
        }
    }
}
