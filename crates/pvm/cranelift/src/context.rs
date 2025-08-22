//! Translator context

use crate::{context_offsets, Translator};
use cranelift::prelude::*;

impl Translator<'_> {
    /// Initialize context
    pub fn init_context(&mut self, ctx: Value) {
        self.init_registers(ctx);
        self.init_memory(ctx);
    }

    /// Initialize memory pointer
    pub fn init_memory(&mut self, ctx: Value) {
        let memory_ptr_offset = self
            .builder
            .ins()
            .iconst(types::I64, context_offsets::MEMORY_PTR_OFFSET as i64);
        let memory_ptr_addr = self.builder.ins().iadd(ctx, memory_ptr_offset);
        self.memory = self
            .builder
            .ins()
            .load(types::I64, MemFlags::new(), memory_ptr_addr, 0);
    }

    /// Initialize registers from context
    pub fn init_registers(&mut self, ctx_ptr: Value) {
        for i in 0..pvm::REGISTER_COUNT {
            let var = Variable::new(i);
            self.builder.declare_var(var, types::I64);

            // Load register from context
            let offset = self.builder.ins().iconst(types::I64, (i * 8) as i64);
            let addr = self.builder.ins().iadd(ctx_ptr, offset);
            let val = self
                .builder
                .ins()
                .load(types::I64, MemFlags::trusted(), addr, 0);
            self.builder.def_var(var, val);
            self.registers.insert(i as u8, var);
        }
    }
}
