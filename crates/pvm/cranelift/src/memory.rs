//! Memory related operations
//!
//!
//! TODO: support static memory when the calculated memory size is less than 1 MB.

use crate::{control::result, offsets, Translator};
use anyhow::Result;
use cranelift::prelude::*;

/// Page size as a power of 2 (2^12 = 4096)
pub const PAGE_SHIFT: u8 = 12;

/// Number of bits per u64 word for bitmap operations
pub const BITS_PER_WORD: u8 = 64;

/// Log2 of bits per u64 word (2^6 = 64)
pub const BITS_PER_WORD_SHIFT: u8 = 6;

/// Log2 of bytes per u64 (2^3 = 8 bytes)
pub const BYTES_PER_U64_SHIFT: u8 = 3;

/// Memory access permissions
pub mod access {
    pub const MUTABLE: u8 = 0;
    pub const IMMUTABLE: u8 = 1;
    pub const INACCESSIBLE: u8 = 2;
}

impl Translator<'_> {
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

    /// Check if a page is allocated and writable by consulting the page bitmap and access array
    pub fn check_page_allocated_and_writable(
        &mut self,
        ctx_ptr: Value,
        page_num: Value,
    ) -> Result<Value, anyhow::Error> {
        // Get page bitmap and access pointers from context
        let bitmap_offset = self
            .builder
            .ins()
            .iconst(types::I64, offsets::PAGE_BITMAP_OFFSET as i64);
        let bitmap_ptr_addr = self.builder.ins().iadd(ctx_ptr, bitmap_offset);
        let bitmap_ptr = self
            .builder
            .ins()
            .load(types::I64, MemFlags::new(), bitmap_ptr_addr, 0);

        let access_offset = self
            .builder
            .ins()
            .iconst(types::I64, offsets::PAGE_ACCESS_OFFSET as i64);
        let access_ptr_addr = self.builder.ins().iadd(ctx_ptr, access_offset);
        let access_ptr = self
            .builder
            .ins()
            .load(types::I64, MemFlags::new(), access_ptr_addr, 0);

        // Check if page is allocated (bit set in bitmap)
        // Use bit shifts for faster bitmap indexing
        let shift_bits = self
            .builder
            .ins()
            .iconst(types::I64, BITS_PER_WORD_SHIFT as i64);
        let word_idx = self.builder.ins().ushr(page_num, shift_bits);
        let mask = self
            .builder
            .ins()
            .iconst(types::I64, (BITS_PER_WORD - 1) as i64);
        let bit_idx = self.builder.ins().band(page_num, mask);

        // Load the bitmap word
        // Use bit shift for word offset: 8 bytes per u64
        let byte_shift = self
            .builder
            .ins()
            .iconst(types::I64, BYTES_PER_U64_SHIFT as i64);
        let word_offset = self.builder.ins().ishl(word_idx, byte_shift);
        let word_addr = self.builder.ins().iadd(bitmap_ptr, word_offset);
        let bitmap_word = self
            .builder
            .ins()
            .load(types::I64, MemFlags::new(), word_addr, 0);

        // Check if the bit is set (page allocated)
        let one_val = self.builder.ins().iconst(types::I64, 1);
        let bit_mask = self.builder.ins().ishl(one_val, bit_idx);
        let bit_value = self.builder.ins().band(bitmap_word, bit_mask);
        let zero_val = self.builder.ins().iconst(types::I64, 0);
        let page_allocated = self
            .builder
            .ins()
            .icmp(IntCC::NotEqual, bit_value, zero_val);

        // Check if page is writable (access == MUTABLE)
        let access_addr = self.builder.ins().iadd(access_ptr, page_num);
        let access_byte = self
            .builder
            .ins()
            .load(types::I8, MemFlags::new(), access_addr, 0);
        let mutable_byte = self.builder.ins().iconst(types::I8, access::MUTABLE as i64);
        let page_writable = self
            .builder
            .ins()
            .icmp(IntCC::Equal, access_byte, mutable_byte);

        // Page is valid if both allocated and writable
        let page_valid = self.builder.ins().band(page_allocated, page_writable);
        Ok(page_valid)
    }

    /// Generate Cranelift IR to check page boundaries before store operations
    /// Uses stored context pointer and simple boundary logic matching interpreter
    pub fn check_store_boundaries(&mut self, address: Value, size_bytes: u32) -> Result<()> {
        let ctx_ptr = self.ctx_ptr;
        let start_page = self.get_page_number(address);
        let size_minus_one = self
            .builder
            .ins()
            .iconst(types::I64, (size_bytes - 1) as i64);
        let last_byte_addr = self.builder.ins().iadd(address, size_minus_one);
        let end_page = self.get_page_number(last_byte_addr);

        // Create blocks for control flow
        let check_start_page_block = self.builder.create_block();
        let check_end_page_block = self.builder.create_block();
        let continue_block = self.builder.create_block();
        let trap_block = self.builder.create_block();

        // Always check the start page first
        self.builder.ins().jump(check_start_page_block, &[]);
        self.builder.switch_to_block(check_start_page_block);
        let start_page_valid = self.check_page_allocated_and_writable(ctx_ptr, start_page)?;
        self.builder
            .ins()
            .brif(start_page_valid, check_end_page_block, &[], trap_block, &[]);

        // Check end page allocation and writability
        self.builder.switch_to_block(check_end_page_block);
        let end_page_valid = self.check_page_allocated_and_writable(ctx_ptr, end_page)?;
        self.builder
            .ins()
            .brif(end_page_valid, continue_block, &[], trap_block, &[]);

        // Trap block: set page fault result and return
        self.builder.switch_to_block(trap_block);
        self.set_result(result::TRAP);
        self.builder.ins().return_(&[]);

        // Continue block: proceed with store operation
        self.builder.switch_to_block(continue_block);
        self.builder.seal_block(check_start_page_block);
        self.builder.seal_block(check_end_page_block);
        self.builder.seal_block(continue_block);
        self.builder.seal_block(trap_block);
        Ok(())
    }

    /// Calculate page number from address using efficient bit shift
    pub fn get_page_number(&mut self, address: Value) -> Value {
        // Use bit shift for much faster page number calculation
        let shift_amount = self.builder.ins().iconst(types::I64, PAGE_SHIFT as i64);
        self.builder.ins().ushr(address, shift_amount)
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
