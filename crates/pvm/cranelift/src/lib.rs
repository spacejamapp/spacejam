//! PVM Compiler - A Cranelift-based compiler for the Polkadot Virtual Machine

use anyhow::Result;
use cranelift::prelude::*;
use cranelift_codegen::ir;
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

/// Jump variable index
pub const JUMP_VAR: usize = 13;

/// PVM-to-Cranelift translator for block-based JIT compilation
pub struct Translator<'b> {
    /// PVM registers (0 to MAX_REGISTER_INDEX)
    pub registers: HashMap<u8, Variable>,

    /// Jump variable
    pub jump: Variable,

    /// Cranelift function builder
    pub builder: FunctionBuilder<'b>,

    // Map of blocks by start PC
    pub blocks: HashMap<u64, ir::Block>,

    // Jump table for dynamic jumps
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
            jump: Variable::new(JUMP_VAR),
            builder: FunctionBuilder::new(func, ctx),
            blocks: HashMap::new(),
            jump_table: Vec::new(),
            ctx_ptr: Value::new(0),
            memory: Value::new(0),
        })
    }
}
