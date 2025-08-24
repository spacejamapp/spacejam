//! PVM Compiler - A Cranelift-based compiler for the Polkadot Virtual Machine

use anyhow::Result;
use cranelift::prelude::*;
use cranelift_codegen::ir::{self, GlobalValue};
use std::collections::HashMap;
pub use {
    context::{offsets, Context},
    control::result,
};

mod context;
mod control;
mod memory;
mod register;
mod translate;
mod visitor;

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

    /// Read data pointer
    read: GlobalValue,

    /// Write data pointer
    write: GlobalValue,
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
            read: GlobalValue::new(0),
            write: GlobalValue::new(0),
        })
    }

    /// Initialize data pointers
    pub fn data(mut self, read: GlobalValue, write: GlobalValue) -> Self {
        self.read = read;
        self.write = write;
        self
    }
}
