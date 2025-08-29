//! Memory related operations

use crate::{offsets, Translator};
use cranelift::prelude::*;

impl Translator<'_> {
    /// Initialize memory pointer
    pub fn init_memory(&mut self) {
        self.memory = self.builder.ins().load(
            types::I64,
            MemFlags::new(),
            self.ctx,
            offsets::MEMORY_PTR_OFFSET,
        );

        self.heap = self.builder.ins().load(
            types::I64,
            MemFlags::new(),
            self.ctx,
            offsets::HEAP_PTR_OFFSET,
        );
    }

    /// Check if the target memory address is allocated
    ///
    ///     allocated: heap start < target < heap ptr
    /// not allocated: heap ptr   < target < heap end
    pub fn allocated(&mut self, _address: Value) {}

    /// Memory get - load value from memory at address
    pub fn mget(&mut self, address: Value, ty: types::Type) -> Value {
        let maddr = self.builder.ins().iadd(self.memory, address);
        self.builder.ins().load(ty, MemFlags::trusted(), maddr, 0)
    }

    /// Memory set - store value to memory at address
    pub fn mset(&mut self, address: Value, value: Value) {
        let maddr = self.builder.ins().iadd(self.memory, address);
        self.builder
            .ins()
            .store(MemFlags::trusted(), value, maddr, 0);
    }

    /// Memory get with offset - load value from memory at address + offset
    pub fn mget_o(&mut self, address: Value, offset: Value, ty: types::Type) -> Value {
        let maddr = self.builder.ins().iadd(address, offset);
        self.mget(maddr, ty)
    }

    /// Memory set with offset - store value to memory at address + offset
    pub fn mset_o(&mut self, address: Value, offset: Value, value: Value) {
        let maddr = self.builder.ins().iadd(address, offset);
        self.mset(maddr, value)
    }
}
