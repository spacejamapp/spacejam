//! The builder for the PVM interpreter.

use crate::Interpreter;

impl Interpreter {
    /// Set the registers of the interpreter.
    pub fn registers(mut self, value: [u32; 13]) -> Self {
        self.registers = value;
        self
    }

    /// Set the gas of the interpreter.
    pub fn gas(mut self, value: u32) -> Self {
        self.gas = value;
        self
    }
}
