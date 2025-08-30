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
    pub host: BTreeMap<&'static str, FuncRef>,

    /// Jump table for dynamic jumps
    jump: Vec<u64>,

    /// The constants pool
    pool: Pool,
}

impl<'b> Translator<'b> {
    /// Create a new translator with PVM register variables and PC
    pub fn new(func: &'b mut ir::Function, ctx: &'b mut FunctionBuilderContext) -> Result<Self> {
        Ok(Self {
            builder: FunctionBuilder::new(func, ctx),
            blocks: BTreeMap::new(),
            host: BTreeMap::new(),
            jump: Vec::new(),
            pool: Pool {
                ctx: Value::new(0),
                memory: Value::new(0),
                heapp: Value::new(0),
                #[cfg(target_os = "macos")]
                read: Value::new(0)..Value::new(0),
                #[cfg(target_os = "macos")]
                write: Value::new(0)..Value::new(0),
                #[cfg(target_os = "macos")]
                heap: Value::new(0)..Value::new(0),
                #[cfg(target_os = "macos")]
                stack: Value::new(0)..Value::new(0),
                #[cfg(target_os = "macos")]
                args: Value::new(0)..Value::new(0),
            },
        })
    }
}
