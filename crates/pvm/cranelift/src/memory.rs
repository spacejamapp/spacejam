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
        self.writable(maddr, value);
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
        self.writable(maddr, value);
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

/// Throw trap directly here for higher performance.
///
/// NOTE: Currently just verified on my amd64 debian machine, need to keep an
/// eye on other platforms.
#[cfg(not(target_os = "macos"))]
impl Translator<'_> {
    /// Check if the target memory address is readable
    pub fn readable(&mut self, _address: Value, _value: Value) {}

    /// Check if the target memory address is writable
    pub fn writable(&mut self, _address: Value, _value: Value) {}
}

/// We have this inline boundary checks just for macos, since it's XNU kernel
/// has an 'optimization' that don't support MAP_NORESERVE correctly.
///
/// For linux, we keep using the signal for throwing the fault since it's more
/// efficient.
#[cfg(target_os = "macos")]
impl Translator<'_> {
    pub fn readable(&mut self, address: Value, value: Value) {
        let bytes = self.builder.func.dfg.value_type(value).bytes();
        let length = self.builder.ins().iconst(types::I64, bytes as i64);
        let end = self.builder.ins().iadd(address, length);

        // check if the address is unallocated
        let is_unallocated = {
            let is_unallocated = self.is_unallocated(address);
            let is_unallocated_end = self.is_unallocated(end);
            self.builder.ins().bor(is_unallocated, is_unallocated_end)
        };

        // check if the address is reserved
        let is_reserved = {
            let is_reserved = self.is_reserved(address);
            let is_reserved_end = self.is_reserved(end);
            self.builder.ins().bor(is_reserved, is_reserved_end)
        };

        // set up condition
        let is_not_readable = self.builder.ins().bor(is_unallocated, is_reserved);
        let fault = self.builder.create_block();
        let belse = self.builder.create_block();
        self.builder
            .ins()
            .brif(is_not_readable, fault, &[], belse, &[]);

        // If the address is not allocated, return FAULT
        self.builder.switch_to_block(fault);
        self.builder.ins().return_(&[address]);
        self.builder.switch_to_block(belse);
    }

    /// Check if the target memory address is writable
    pub fn writable(&mut self, address: Value, value: Value) {
        let bytes = self.builder.func.dfg.value_type(value).bytes();
        let length = self.builder.ins().iconst(types::I64, bytes as i64);
        let end = self.builder.ins().iadd(address, length);

        // check if the address is in the write range
        let is_write = {
            let is_write = self.is_write(address);
            let is_write_end = self.is_write(end);
            self.builder.ins().band(is_write, is_write_end)
        };

        // check if the address is in the heap range
        let is_heap = {
            let is_heap = self.is_heap(address);
            let is_heap_end = self.is_heap(end);
            self.builder.ins().band(is_heap, is_heap_end)
        };

        // check if the address is in the stack range
        let is_stack = {
            let is_stack = self.is_stack(address);
            let is_stack_end = self.is_stack(end);
            self.builder.ins().band(is_stack, is_stack_end)
        };

        // check if is writable
        let is_writable = {
            let write_or_heap = self.builder.ins().bor(is_write, is_heap);
            self.builder.ins().bor(write_or_heap, is_stack)
        };

        // set up condition
        let fault = self.builder.create_block();
        let then = self.builder.create_block();
        self.builder.ins().brif(is_writable, then, &[], fault, &[]);

        // If the address is not allocated, return FAULT
        self.builder.switch_to_block(fault);
        self.builder.ins().return_(&[address]);
        self.builder.switch_to_block(then);
    }

    /// Check if the target memory address is in the read range
    pub fn is_read(&mut self, address: Value) -> Value {
        let above = self.builder.ins().icmp(
            IntCC::UnsignedGreaterThanOrEqual,
            address,
            self.pool.read.start,
        );
        let below =
            self.builder
                .ins()
                .icmp(IntCC::UnsignedLessThanOrEqual, address, self.pool.read.end);
        self.builder.ins().band(above, below)
    }

    /// Check if the target memory address is in the write range
    pub fn is_write(&mut self, address: Value) -> Value {
        let above = self.builder.ins().icmp(
            IntCC::UnsignedGreaterThanOrEqual,
            address,
            self.pool.write.start,
        );
        let below =
            self.builder
                .ins()
                .icmp(IntCC::UnsignedLessThanOrEqual, address, self.pool.write.end);
        self.builder.ins().band(above, below)
    }

    /// Check if the target memory address is in the heap
    pub fn is_heap(&mut self, address: Value) -> Value {
        let above = self.builder.ins().icmp(
            IntCC::UnsignedGreaterThanOrEqual,
            address,
            self.pool.heap.start,
        );
        let below =
            self.builder
                .ins()
                .icmp(IntCC::UnsignedLessThanOrEqual, address, self.pool.heapp);
        self.builder.ins().band(above, below)
    }

    /// Check if the target memory address is unallocated
    pub fn is_unallocated(&mut self, address: Value) -> Value {
        let above =
            self.builder
                .ins()
                .icmp(IntCC::UnsignedGreaterThanOrEqual, address, self.pool.heapp);
        let below =
            self.builder
                .ins()
                .icmp(IntCC::UnsignedLessThanOrEqual, address, self.pool.heap.end);
        self.builder.ins().band(above, below)
    }

    /// Check if the target memory address is in the stack
    pub fn is_stack(&mut self, address: Value) -> Value {
        let above = self.builder.ins().icmp(
            IntCC::UnsignedGreaterThanOrEqual,
            address,
            self.pool.stack.start,
        );
        let below =
            self.builder
                .ins()
                .icmp(IntCC::UnsignedLessThanOrEqual, address, self.pool.stack.end);
        self.builder.ins().band(above, below)
    }

    /// Check if the target memory address is in the args range
    pub fn is_args(&mut self, address: Value) -> Value {
        let above = self.builder.ins().icmp(
            IntCC::UnsignedGreaterThanOrEqual,
            address,
            self.pool.args.start,
        );
        let below =
            self.builder
                .ins()
                .icmp(IntCC::UnsignedLessThanOrEqual, address, self.pool.args.end);
        self.builder.ins().band(above, below)
    }

    /// Check if the target memory address is reserved
    pub fn is_reserved(&mut self, address: Value) -> Value {
        let above = self.builder.ins().icmp(
            IntCC::UnsignedGreaterThanOrEqual,
            address,
            self.pool.args.end,
        );
        let below = self.builder.ins().icmp(
            IntCC::UnsignedLessThanOrEqual,
            address,
            self.pool.read.start,
        );
        self.builder.ins().bor(above, below)
    }
}
