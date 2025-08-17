//! Translator context

use crate::{constants::PVM_REGISTER_COUNT, Translator};
use anyhow::Result;
use cranelift::prelude::*;

impl<'b> Translator<'b> {
    /// Get context pointer for visitor operations - handles both unified and block-based modes
    pub fn get_context_ptr_for_visitor(&self) -> Value {
        // In unified mode, use the stored context pointer
        self.ctx_ptr
            .expect("Context pointer not initialized in unified mode")
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
