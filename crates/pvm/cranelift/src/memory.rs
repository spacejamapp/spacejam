//! Memory related operations
//!
//!
//! TODO: support static memory when the calculated memory size is less than 1 MB.

use crate::{offsets, Translator};
use cranelift::prelude::*;
use pvm::Memory;

impl Translator<'_> {
    /// Initialize memory pointer
    pub fn init_memory(&mut self, ctx: Value, _memory: &Memory) {
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

    /// Memory get - load value from memory at address
    pub fn mget(&mut self, address: Value, ty: types::Type) -> Value {
        let mem_addr = self.builder.ins().iadd(self.memory, address);
        self.builder
            .ins()
            .load(ty, MemFlags::trusted(), mem_addr, 0)
    }

    /// Memory set - store value to memory at address
    pub fn mset(&mut self, address: Value, value: Value) {
        let mem_addr = self.builder.ins().iadd(self.memory, address);
        self.builder
            .ins()
            .store(MemFlags::trusted(), value, mem_addr, 0);
    }

    /// Memory get with offset - load value from memory at address + offset
    pub fn mget_o(&mut self, address: Value, offset: Value, ty: types::Type) -> Value {
        let addr_with_offset = self.builder.ins().iadd(address, offset);
        let mem_addr = self.builder.ins().iadd(self.memory, addr_with_offset);
        self.builder
            .ins()
            .load(ty, MemFlags::trusted(), mem_addr, 0)
    }

    /// Memory set with offset - store value to memory at address + offset
    pub fn mset_o(&mut self, address: Value, offset: Value, value: Value) {
        let addr_with_offset = self.builder.ins().iadd(address, offset);
        let mem_addr = self.builder.ins().iadd(self.memory, addr_with_offset);
        self.builder
            .ins()
            .store(MemFlags::trusted(), value, mem_addr, 0);
    }
}
