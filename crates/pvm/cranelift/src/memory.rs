//! Memory related operations

use crate::Translator;
use cranelift::prelude::*;

impl Translator<'_> {
    /// Memory get with immediate offset
    pub fn mget(&mut self, address: Value, offset_imm: i64, ty: types::Type) -> Value {
        let offset = self.builder.ins().iadd_imm(address, offset_imm);
        let maddr = self.builder.ins().iadd(self.pool.memory, offset);
        self.builder.ins().load(ty, MemFlags::trusted(), maddr, 0)
    }

    /// Memory get with immediate address - optimized for constant addresses
    pub fn mget_imm(&mut self, address_imm: i64, ty: types::Type) -> Value {
        let maddr = self.builder.ins().iadd_imm(self.pool.memory, address_imm);
        self.builder.ins().load(ty, MemFlags::trusted(), maddr, 0)
    }

    /// Memory set with immediate offset
    pub fn mset(&mut self, address: Value, offset_imm: i64, value: Value) {
        let offset = self.builder.ins().iadd_imm(address, offset_imm);
        let maddr = self.builder.ins().iadd(self.pool.memory, offset);
        self.builder
            .ins()
            .store(MemFlags::trusted(), value, maddr, 0);
    }

    /// Memory set with immediate address - optimized for constant addresses  
    pub fn mset_imm(&mut self, address_imm: i64, value: Value) {
        let maddr = self.builder.ins().iadd_imm(self.pool.memory, address_imm);
        self.builder
            .ins()
            .store(MemFlags::trusted(), value, maddr, 0);
    }

    /// Store immediate value at immediate address
    pub fn mset_iimm(&mut self, address_imm: i64, value_imm: i64, ty: types::Type) {
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

impl Translator<'_> {
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
        self.builder
            .ins()
            .call(self.host["mset"], &[self.pool.ctx, address, value, clen]);
    }
}
