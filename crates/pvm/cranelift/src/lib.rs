//! PVM Compiler - A Cranelift-based compiler for the Polkadot Virtual Machine

use anyhow::Result;
use cranelift::prelude::*;
use cranelift_codegen::ir::{self, FuncRef};
use std::collections::BTreeMap;
pub use {
    context::{offsets, Pool},
    exit::Exit,
};

mod context;
mod control;
mod exit;
mod math;
mod memory;
mod register;
mod translate;
mod visitor;

/// PVM-to-Cranelift translator for block-based JIT compilation
pub struct Translator<'b> {
    /// Cranelift function builder
    pub builder: FunctionBuilder<'b>,

    /// Map of blocks by start PC
    pub blocks: BTreeMap<u64, ir::Block>,

    /// The host call function
    pub host: BTreeMap<String, FuncRef>,

    /// If the translator is used for testing
    testing: bool,

    /// Jump table for dynamic jumps
    jump: Vec<u64>,

    /// Runtime jump table for br_table instruction (cached)
    rt_jump_table: ir::JumpTable,

    /// The constants pool
    pool: Pool,

    /// The memory info
    #[cfg(target_os = "macos")]
    pub memory: pvm::MemoryInfo,
}

impl<'b> Translator<'b> {
    /// Create a new translator with PVM register variables and PC
    pub fn new(func: &'b mut ir::Function, ctx: &'b mut FunctionBuilderContext) -> Result<Self> {
        let testing = std::env::var("PVM_TESTING").map_or(false, |v| v == "true");
        Ok(Self {
            builder: FunctionBuilder::new(func, ctx),
            blocks: BTreeMap::new(),
            host: BTreeMap::new(),
            testing,
            jump: Vec::new(),
            rt_jump_table: ir::JumpTable::new(0),
            pool: Pool::default(),
            #[cfg(target_os = "macos")]
            memory: pvm::MemoryInfo::default(),
        })
    }
}
