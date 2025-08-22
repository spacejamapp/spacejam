//! PVM Compiler - A Cranelift-based compiler for the Polkadot Virtual Machine

use anyhow::Result;
use cranelift::prelude::*;
use cranelift_codegen::ir;
use std::collections::HashMap;
pub use {
    control::result,
    memory::{access, BITS_PER_WORD},
};

mod context;
mod control;
mod memory;
mod register;
mod translate;
mod visitor;

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

/// PVM-to-Cranelift translator for block-based JIT compilation
pub struct Translator<'b> {
    /// PVM registers (0 to MAX_REGISTER_INDEX)
    pub registers: HashMap<u8, Variable>,

    /// Cranelift function builder
    pub builder: FunctionBuilder<'b>,

    // Map of blocks by start PC
    pub blocks: HashMap<u64, ir::Block>,

    // Jump table for dynamic jumps (djump)
    jump_table: Vec<u64>,

    // Context pointer for boundary checking and runtime operations
    ctx_ptr: Value,

    /// ssv for memory pointer
    memory: Value,
}

impl<'b> Translator<'b> {
    /// Create a new translator with PVM register variables and PC
    pub fn new(func: &'b mut ir::Function, ctx: &'b mut FunctionBuilderContext) -> Result<Self> {
        Ok(Self {
            registers: HashMap::new(),
            builder: FunctionBuilder::new(func, ctx),
            jump_table: Vec::new(),
            blocks: HashMap::new(),
            ctx_ptr: Value::new(0),
            memory: Value::new(0),
        })
    }
}
