//! Register interfaces

use crate::Interpreter;

impl Interpreter {
    /// set the register with the given value.
    pub fn rset(&mut self, reg: u8, value: u64) {
        self.registers[reg as usize] = value;

        match reg {
            0 => tracing::trace!("ra = 0x{:x}", value),
            1 => tracing::trace!("sp = 0x{:x}", value),
            5 => tracing::trace!("s0 = 0x{:x}", value),
            6 => tracing::trace!("s1 = 0x{:x}", value),
            7 => tracing::trace!("a0 = 0x{:x}", value),
            8 => tracing::trace!("a1 = 0x{:x}", value),
            9 => tracing::trace!("a2 = 0x{:x}", value),
            10 => tracing::trace!("a3 = 0x{:x}", value),
            11 => tracing::trace!("a4 = 0x{:x}", value),
            _ => {}
        }
    }

    /// get the register value.
    pub fn rget(&self, reg: u8) -> u64 {
        self.registers[reg as usize]
    }
}
