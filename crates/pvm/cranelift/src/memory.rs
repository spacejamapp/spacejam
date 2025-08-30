//! Memory related operations

use crate::Translator;
use cranelift::prelude::*;

impl Translator<'_> {
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

    /// Memory get with immediate offset - optimized for constant offsets
    pub fn mget_o_imm(&mut self, address: Value, offset_imm: i64, ty: types::Type) -> Value {
        let maddr = self.builder.ins().iadd_imm(address, offset_imm);
        self.mget(maddr, ty)
    }

    /// Memory set with immediate offset - optimized for constant offsets
    pub fn mset_o_imm(&mut self, address: Value, offset_imm: i64, value: Value) {
        let maddr = self.builder.ins().iadd_imm(address, offset_imm);
        self.mset(maddr, value)
    }

    /// Memory get with immediate address - optimized for constant addresses
    pub fn mget_imm(&mut self, address_imm: i64, ty: types::Type) -> Value {
        let maddr = self.builder.ins().iadd_imm(self.pool.memory, address_imm);
        self.builder.ins().load(ty, MemFlags::trusted(), maddr, 0)
    }

    /// Memory set with immediate address - optimized for constant addresses  
    pub fn mset_imm(&mut self, address_imm: i64, value: Value) {
        let maddr = self.builder.ins().iadd_imm(self.pool.memory, address_imm);
        self.builder
            .ins()
            .store(MemFlags::trusted(), value, maddr, 0);
    }

    /// Store immediate value at immediate address
    pub fn mset_imm_imm(&mut self, address_imm: i64, value_imm: i64, ty: types::Type) {
        let maddr = self.builder.ins().iadd_imm(self.pool.memory, address_imm);
        let value = self.builder.ins().iconst(ty, value_imm);
        let write_value =
            if ty.bits() < 64 && self.builder.func.dfg.value_type(value).bits() > ty.bits() {
                self.builder.ins().ireduce(ty, value)
            } else {
                value
            };
        self.builder
            .ins()
            .store(MemFlags::trusted(), write_value, maddr, 0);
    }
}
