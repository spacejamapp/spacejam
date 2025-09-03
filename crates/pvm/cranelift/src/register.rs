//! Register related operations

use crate::Translator;
use cranelift::prelude::*;

impl Translator<'_> {
    /// get register value
    pub fn rget(&mut self, reg: u8) -> Value {
        self.pool.registers[reg as usize]
    }

    /// set register value
    pub fn rset(&mut self, reg: u8, value: Value) {
        self.pool.registers[reg as usize] = value;
    }
}
