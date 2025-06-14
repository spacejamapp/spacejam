//! Register interfaces

use crate::Interpreter;
use pvm::Accounts;

impl<R: Accounts> Interpreter<R> {
    /// set the register with the given value.
    pub fn rset(&mut self, reg: u8, value: u64) {
        self.registers[reg as usize] = value;
    }

    /// get the register value.
    pub fn rget(&self, reg: u8) -> u64 {
        self.registers[reg as usize]
    }
}
