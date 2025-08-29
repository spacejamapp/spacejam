//! PVM Compiler - A Cranelift-based compiler for the Polkadot Virtual Machine

use crate::context::Pool;
use anyhow::Result;
use cranelift::prelude::*;
use cranelift_codegen::ir::{self, FuncRef};
use std::collections::BTreeMap;
pub use {context::offsets, control::result};

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
    pub registers: BTreeMap<u8, Variable>,

    /// Jump variable
    pub jump: Variable,

    /// Cranelift function builder
    pub builder: FunctionBuilder<'b>,

    // Map of blocks by start PC
    pub blocks: BTreeMap<u64, ir::Block>,

    /// The host call function
    pub host: BTreeMap<&'static str, FuncRef>,

    // Jump table for dynamic jumps
    jump_table: Vec<u64>,

    pool: Pool,
}

impl<'b> Translator<'b> {
    /// Create a new translator with PVM register variables and PC
    pub fn new(func: &'b mut ir::Function, ctx: &'b mut FunctionBuilderContext) -> Result<Self> {
        Ok(Self {
            registers: BTreeMap::new(),
            jump: Variable::new(JUMP_VAR),
            builder: FunctionBuilder::new(func, ctx),
            blocks: BTreeMap::new(),
            host: BTreeMap::new(),
            jump_table: Vec::new(),
            pool: Pool {
                ctx: Value::new(0),
                memory: Value::new(0),
                heap: Value::new(0),
                hrange: Value::new(0)..Value::new(0),
            },
        })
    }
}
