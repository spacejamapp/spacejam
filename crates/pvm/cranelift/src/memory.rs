//! Memory related operations

use crate::Translator;
use cranelift::prelude::*;

impl Translator<'_> {
    /// Check if the target memory address is allocated
    ///
    /// not allocated: heap ptr < target < heap end
    pub fn allocated(&mut self, address: Value) {
        let above = self
            .builder
            .ins()
            .icmp(IntCC::UnsignedGreaterThan, address, self.pool.heap);
        let below = self
            .builder
            .ins()
            .icmp(IntCC::UnsignedLessThan, address, self.pool.hrange.end);
        let is_not_allocated = self.builder.ins().band(above, below);

        // set up condition
        let fault = self.builder.create_block();
        let then = self.builder.create_block();
        self.builder
            .ins()
            .brif(is_not_allocated, fault, &[], then, &[]);

        // If the address is not allocated, return FAULT
        self.builder.switch_to_block(fault);
        self.builder.ins().return_(&[address]);
        self.builder.switch_to_block(then);
    }

    /// Memory get - load value from memory at address
    pub fn mget(&mut self, address: Value, ty: types::Type) -> Value {
        let maddr = self.builder.ins().iadd(self.pool.memory, address);
        self.builder.ins().load(ty, MemFlags::trusted(), maddr, 0)
    }

    /// Memory set - store value to memory at address
    pub fn mset(&mut self, address: Value, value: Value) {
        let maddr = self.builder.ins().iadd(self.pool.memory, address);
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
