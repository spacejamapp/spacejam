//! Translator context

use crate::{result, Translator};
use cranelift::prelude::*;

/// ExtendedContext memory layout offsets
pub mod offsets {
    /// Size of register array in bytes
    pub const REGISTERS_SIZE: usize = pvm::REGISTER_COUNT * 8;

    /// Offset to PC field (after registers)
    pub const PC_OFFSET: usize = REGISTERS_SIZE;

    /// Offset to memory pointer (after registers + PC)
    pub const MEMORY_PTR_OFFSET: usize = REGISTERS_SIZE + 8;

    /// Offset to page bitmap pointer (after registers + PC + memory_ptr)
    pub const PAGE_BITMAP_OFFSET: usize = REGISTERS_SIZE + 8 + 8;

    /// Offset to page access array pointer (after registers + PC + memory_ptr + page_bitmap)
    pub const PAGE_ACCESS_OFFSET: usize = REGISTERS_SIZE + 8 + 8 + 8;

    /// Offset to execution result (after registers + PC + memory_ptr + page_bitmap + page_access)
    pub const RESULT_OFFSET: usize = REGISTERS_SIZE + 8 + 8 + 8 + 8;

    /// Offset to dynamic jump target pointer (after registers + PC + memory_ptr + page_bitmap + page_access + result)
    pub const JUMP_TABLE_OFFSET: usize = REGISTERS_SIZE + 8 + 8 + 8 + 8 + 8;
}

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
            .iconst(types::I64, offsets::MEMORY_PTR_OFFSET as i64);
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

    /// get result from the context
    pub fn jump(&mut self) -> Value {
        let offset = self
            .builder
            .ins()
            .iconst(types::I64, offsets::JUMP_TABLE_OFFSET as i64);
        let addr = self.builder.ins().iadd(self.ctx_ptr, offset);
        self.builder
            .ins()
            .load(types::I64, MemFlags::trusted(), addr, 0)
    }

    /// set dynamic jump to the context
    pub fn set_jump(&mut self, target: Value) {
        self.set_result(result::JUMP_INDIRECT);
        let data_addr = self
            .builder
            .ins()
            .iconst(types::I64, offsets::JUMP_TABLE_OFFSET as i64);
        self.builder
            .ins()
            .store(MemFlags::new(), target, data_addr, 0);
    }

    /// set pc to the context
    pub fn set_pc(&mut self, pc: u64) {
        let pc_offset = self
            .builder
            .ins()
            .iconst(types::I64, offsets::PC_OFFSET as i64);
        let pc_addr = self.builder.ins().iadd(self.ctx_ptr, pc_offset);
        let pc_val = self.builder.ins().iconst(types::I64, pc as i64);
        self.builder
            .ins()
            .store(MemFlags::new(), pc_val, pc_addr, 0);
    }

    /// set result to the context
    pub fn set_result(&mut self, result: u64) {
        let offset = self
            .builder
            .ins()
            .iconst(types::I64, offsets::RESULT_OFFSET as i64);
        let addr = self.builder.ins().iadd(self.ctx_ptr, offset);
        let val = self.builder.ins().iconst(types::I64, result as i64);
        self.builder.ins().store(MemFlags::new(), val, addr, 0);
    }
}
