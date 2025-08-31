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
    pub fn mget(&mut self, address: Value, offset: i64, ty: types::Type) -> Value {
        let address = self.builder.ins().iadd_imm(address, offset);
        self.mload(ty, address)
    }

    /// Memory get with immediate address - optimized for constant addresses
    pub fn mget_imm(&mut self, address: i64, ty: types::Type) -> Value {
        let maddr = self.maddr(address);
        let maddr = self.builder.ins().iadd_imm(self.pool.memory, maddr);
        self.builder.ins().load(ty, MemFlags::trusted(), maddr, 0)
    }

    /// Memory set with immediate offset
    pub fn mset(&mut self, address: Value, offset: i64, value: Value) {
        let address = self.builder.ins().iadd_imm(address, offset);
        self.mstore(address, value)
    }

    /// Memory set with immediate address - optimized for constant addresses  
    pub fn mset_imm(&mut self, address: i64, value: Value) {
        let maddr = self.maddr(address);
        self.builder
            .ins()
            .store(MemFlags::trusted(), value, self.pool.memory, maddr as i32);
    }

    /// Store immediate value at immediate address
    pub fn mset_iimm(&mut self, address: i64, value: i64, ty: types::Type) {
        let maddr = self.maddr(address);
        let value = self.builder.ins().iconst(ty, value);
        self.builder
            .ins()
            .store(MemFlags::trusted(), value, self.pool.memory, maddr as i32);
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

    /// Convert the given address to the real address in memory
    pub fn maddr(&mut self, address: i64) -> i64 {
        let start = address as u32;
        if start >= self.memory.read.start
            && start < self.memory.write.start.max(self.memory.read.end)
        {
            return (start - self.memory.read.start) as i64;
        }

        // now the pointer is at the start of the write area
        let mut ptr = self.memory.read.len() as u32;
        if start >= self.memory.write.start
            && start < self.memory.heap.start.max(self.memory.write.end)
        {
            return (ptr + start - self.memory.write.start) as i64;
        }

        // now the pointer is at the start of the stack area
        ptr += self.memory.write.len() as u32;
        if start >= self.memory.stack.start
            && start < self.memory.args.start.max(self.memory.stack.end)
        {
            return (ptr + start - self.memory.stack.start) as i64;
        }

        // now the pointer is at the start of the args area
        ptr += self.memory.stack.len() as u32;
        if start >= self.memory.args.start
            && start < self.memory.args.start.max(self.memory.args.end)
        {
            return (ptr + start - self.memory.args.start) as i64;
        }

        // now the pointer is at the start of the heap area
        ptr += self.memory.args.len() as u32;
        (ptr + start - self.memory.heap.start) as i64
    }
}
