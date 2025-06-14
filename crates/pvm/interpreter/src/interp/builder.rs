//! The builder for the PVM interpreter.

use crate::{Interpreter, Memory, Register};
use std::marker::PhantomData;

impl Interpreter {
    /// Set the registers of the interpreter.
    pub fn registers(mut self, value: [Register; 13]) -> Self {
        self.registers = value;
        self
    }

    /// Set the memory of the interpreter.
    pub fn memory(mut self, value: Memory) -> Self {
        self.memory = value;
        self
    }

    /// Set the gas of the interpreter.
    pub fn gas(mut self, value: u64) -> Self {
        self.gas = value;
        self
    }

    /// Set the program counter of the interpreter.
    pub fn pc(mut self, value: usize) -> Self {
        self.pc = value;
        self
    }

    /// Set the jump table of the interpreter.
    pub fn table(mut self, value: Vec<u64>) -> Self {
        self.table = value;
        self
    }
}
