//! Register related operations

use crate::Translator;
use cranelift::prelude::*;

impl Translator<'_> {
    /// get register value
    pub fn rget(&mut self, reg: u8) -> Value {
        let reg_var = self.registers[&reg];
        self.builder.use_var(reg_var)
    }

    /// set register value
    pub fn rset(&mut self, reg: u8, value: Value) {
        let reg_var = self.registers[&reg];
        self.builder.def_var(reg_var, value);
    }

    // Save registers to context
    pub fn save_registers(&mut self) {
        let ctx_ptr = self.ctx_ptr;
        for i in 0..pvm::REGISTER_COUNT {
            let reg_var = self.registers[&(i as u8)];
            let reg_val = self.builder.use_var(reg_var);
            let offset = self.builder.ins().iconst(types::I64, (i * 8) as i64);
            let addr = self.builder.ins().iadd(ctx_ptr, offset);
            self.builder.ins().store(MemFlags::new(), reg_val, addr, 0);
        }
    }
}
