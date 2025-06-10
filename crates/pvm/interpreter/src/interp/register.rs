//! Register interfaces

use crate::Interpreter;

impl Interpreter {
    /// set the register with the given value.
    pub fn rset(&mut self, reg: u8, value: u64) {
        self.registers[reg as usize] = value;
    }

    /// get the register value.
    pub fn rget(&self, reg: u8) -> u64 {
        self.registers[reg as usize]
    }
}

/// get the register name.
pub fn fmt(reg: u8) -> &'static str {
    match reg {
        0 => "ra",
        1 => "sp",
        5 => "s0",
        6 => "s1",
        7 => "a0",
        8 => "a1",
        9 => "a2",
        10 => "a3",
        11 => "a4",
        _ => "unknown",
    }
}
