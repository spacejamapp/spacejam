//! Translator context

use crate::Translator;
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
    pub const PAGE_BITMAP_OFFSET: usize = MEMORY_PTR_OFFSET + 8;

    /// Offset to page access array pointer (after registers + PC + memory_ptr + page_bitmap)
    pub const PAGE_ACCESS_OFFSET: usize = PAGE_BITMAP_OFFSET + 8;

    /// Offset to execution result (after registers + PC + memory_ptr + page_bitmap + page_access)
    pub const RESULT_OFFSET: usize = PAGE_ACCESS_OFFSET + 8;

    /// Offset to dynamic jump target pointer (after registers + PC + memory_ptr + page_bitmap + page_access + result)
    pub const JUMP_TARGET_OFFSET: usize = RESULT_OFFSET + 8;
}

/// The context of the translator.
pub struct Context {
    /// The registers of the context.
    pub registers: [u64; pvm::REGISTER_COUNT],

    // legacy fields, will be removed soon!
    pub pc: u64,
    pub memory_ptr: *mut u8,
    pub page_bitmap: *const u64,
    pub page_access: *const u8,
    pub result: u64,
    pub jump_target: u64,
}

impl Translator<'_> {
    /// Initialize context
    pub fn init_context(&mut self, ctx: Value) {
        self.init_registers(ctx);
        self.init_memory(ctx);
    }
}
