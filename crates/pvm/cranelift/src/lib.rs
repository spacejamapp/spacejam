//! PVM Compiler - A Cranelift-based compiler for the Polkadot Virtual Machine

pub use translator::{Block, Code, Translator};

pub mod translator;

/// Page size as a power of 2 (2^12 = 4096)
pub const PAGE_SHIFT: u8 = 12;

/// Number of bits per u64 word for bitmap operations
pub const BITS_PER_WORD: u8 = 64;

/// Log2 of bits per u64 word (2^6 = 64)
pub const BITS_PER_WORD_SHIFT: u8 = 6;

/// Log2 of bytes per u64 (2^3 = 8 bytes)
pub const BYTES_PER_U64_SHIFT: u8 = 3;

/// Extra pages to allocate in access array for boundary checking safety
pub const EXTRA_PAGES_MARGIN: u32 = 64;

/// Linear memory size for JIT execution (1MB)
pub const LINEAR_MEMORY_SIZE: usize = 0x100000;

/// Maximum register index (0-12, so 12 is the maximum valid index)
pub const MAX_REGISTER_INDEX: u8 = pvm::REGISTER_COUNT as u8 - 1;

/// Execution result discriminant values
pub mod result {
    pub const CONTINUE: u64 = 0;
    pub const HALT: u64 = 2;
    pub const TRAP: u64 = 3;
    pub const JUMP_INDIRECT: u64 = 4;
}

/// Memory access permissions
pub mod access {
    pub const MUTABLE: u8 = 0;
    pub const IMMUTABLE: u8 = 1;
    pub const INACCESSIBLE: u8 = 2;
}

/// ExtendedContext memory layout offsets
pub mod context_offsets {
    /// Size of register array in bytes
    pub const REGISTERS_SIZE: usize = pvm::REGISTER_COUNT * 8;

    /// Offset to PC field (after registers)
    pub const PC_OFFSET: usize = REGISTERS_SIZE;

    /// Offset to memory pointer (after registers + PC)
    pub const MEMORY_PTR_OFFSET: usize = REGISTERS_SIZE + 8;

    /// Offset to page bitmap pointer (after registers + PC + memory_ptr)
    pub const PAGE_BITMAP_OFFSET: usize = REGISTERS_SIZE + 8 + 8;

    /// Offset to page access array pointer (after registers + PC + memory_ptr + page_bitmap)
    pub const PAGE_ACCESS_OFFSET: usize = REGISTERS_SIZE + 8 + 8 + 8;

    /// Offset to execution result (after registers + PC + memory_ptr + page_bitmap + page_access)
    pub const RESULT_OFFSET: usize = REGISTERS_SIZE + 8 + 8 + 8 + 8;
}
