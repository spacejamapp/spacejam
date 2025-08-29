//! Register related operations

use crate::Translator;
use cranelift::prelude::*;

impl Translator<'_> {
    /// get register value
    pub fn rget(&mut self, reg: u8) -> Value {
        let offset = (reg as i32) * 8;
        self.builder
            .ins()
            .load(types::I64, MemFlags::trusted(), self.pool.ctx, offset)
    }

    /// set register value
    pub fn rset(&mut self, reg: u8, value: Value) {
        let offset = (reg as i32) * 8;
        self.builder
            .ins()
            .store(MemFlags::new(), value, self.pool.ctx, offset);
    }
}
