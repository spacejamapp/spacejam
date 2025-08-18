//! Translator context

use crate::{constants::PVM_REGISTER_COUNT, Translator};
use anyhow::Result;
use cranelift::prelude::*;

impl Translator<'_> {
    /// Get context pointer for visitor operations
    pub fn get_context_ptr_for_visitor(&self) -> Value {
        self.ctx_ptr.expect("Context pointer not initialized")
    }

    // Save registers to context
    pub fn save_registers(&mut self) -> Result<()> {
        let ctx_ptr = self
            .get_context_ptr()
            .expect("Context pointer not initialized");

        for i in 0..PVM_REGISTER_COUNT {
            let reg_var = self.registers[&(i as u8)];
            let reg_val = self.builder.use_var(reg_var);
            let offset = self.builder.ins().iconst(types::I64, (i * 8) as i64);
            let addr = self.builder.ins().iadd(ctx_ptr, offset);
            self.builder.ins().store(MemFlags::new(), reg_val, addr, 0);
        }
        Ok(())
    }
}
