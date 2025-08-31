//! Memory related operations

use crate::Translator;
use cranelift::prelude::*;

#[cfg(not(target_os = "macos"))]
impl Translator<'_> {
    /// Memory get with immediate offset
    pub fn mget(&mut self, address: Value, offset: i64, ty: types::Type) -> Value {
        let offset = self.builder.ins().iadd_imm(address, offset);
        let maddr = self.builder.ins().iadd(self.pool.memory, offset);
        self.builder.ins().load(ty, MemFlags::trusted(), maddr, 0)
    }

    /// Memory get with immediate address - optimized for constant addresses
    pub fn mget_imm(&mut self, address: i64, ty: types::Type) -> Value {
        let maddr = self.builder.ins().iadd_imm(self.pool.memory, address);
        self.builder.ins().load(ty, MemFlags::trusted(), maddr, 0)
    }

    /// Memory set with immediate offset
    pub fn mset(&mut self, address: Value, offset: i64, value: Value) {
        let offset = self.builder.ins().iadd_imm(address, offset);
        let maddr = self.builder.ins().iadd(self.pool.memory, offset);
        self.builder
            .ins()
            .store(MemFlags::trusted(), value, maddr, 0);
    }

    /// Memory set with immediate address - optimized for constant addresses  
    pub fn mset_imm(&mut self, address: i64, value: Value) {
        let maddr = self.builder.ins().iadd_imm(self.pool.memory, address);
        self.builder
            .ins()
            .store(MemFlags::trusted(), value, maddr, 0);
    }

    /// Store immediate value at immediate address
    pub fn mset_iimm(&mut self, address: i64, value: i64, ty: types::Type) {
        let maddr = self.builder.ins().iadd_imm(self.pool.memory, address);
        let value = self.builder.ins().iconst(ty, value);
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

#[cfg(target_os = "macos")]
impl Translator<'_> {
    /// Memory get with immediate offset
    ///
    /// TODO: for imm operations, we might not need to load the address
    pub fn mget(&mut self, address: Value, offset_imm: i64, ty: types::Type) -> Value {
        let offset = self.builder.ins().iadd_imm(address, offset_imm);
        self.mload(ty, offset)
    }

    /// Memory get with immediate address - optimized for constant addresses
    ///
    /// TODO: for imm operations, we might not need to load the address
    pub fn mget_imm(&mut self, address_imm: i64, ty: types::Type) -> Value {
        let offset = self.builder.ins().iconst(types::I64, address_imm);
        self.mload(ty, offset)
    }

    /// Memory set with immediate offset
    ///
    /// TODO: for imm operations, we might not need to load the address
    pub fn mset(&mut self, address: Value, offset_imm: i64, value: Value) {
        let offset = self.builder.ins().iadd_imm(address, offset_imm);
        self.mstore(offset, value)
    }

    /// Memory set with immediate address - optimized for constant addresses  
    ///
    /// TODO: for imm operations, we don't need to load the address
    pub fn mset_imm(&mut self, address: i64, value: Value) {
        let offset = self.builder.ins().iconst(types::I64, address);
        self.mstore(offset, value)
    }

    /// Store immediate value at immediate address
    ///
    /// TODO: for imm operations, we might not need to load the address
    pub fn mset_iimm(&mut self, address: i64, value: i64, ty: types::Type) {
        let offset = self.builder.ins().iconst(types::I64, address);
        let value = self.builder.ins().iconst(ty, value);
        self.mstore(offset, value)
    }

    /// Memory load abi
    pub fn mload(&mut self, ty: types::Type, address: Value) -> Value {
        let length = match ty {
            types::I8 => 1,
            types::I16 => 2,
            types::I32 => 4,
            types::I64 => 8,
            _ => panic!("invalid type"),
        };
        let clen = self.builder.ins().iconst(types::I8, length);
        let inst = self
            .builder
            .ins()
            .call(self.host["mget"], &[self.pool.ctx, address, clen]);
        let value = self.builder.inst_results(inst)[0];
        if length != 8 {
            self.builder.ins().ireduce(ty, value)
        } else {
            value
        }
    }

    /// Memory store abi
    pub fn mstore(&mut self, address: Value, value: Value) {
        let length = self.builder.func.dfg.value_type(value).bytes();
        let clen = self.builder.ins().iconst(types::I8, length as i64);
        let value = match self.builder.func.dfg.value_type(value).bytes() {
            1 => self.builder.ins().uextend(types::I64, value),
            2 => self.builder.ins().uextend(types::I64, value),
            4 => self.builder.ins().uextend(types::I64, value),
            8 => value,
            _ => panic!("invalid value length"),
        };
        self.builder
            .ins()
            .call(self.host["mset"], &[self.pool.ctx, address, value, clen]);
    }
}
