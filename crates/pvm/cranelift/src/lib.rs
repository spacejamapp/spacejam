//! PVM Compiler - A Cranelift-based compiler for the Polkadot Virtual Machine

use crate::masm::MacroBlocks;
use anyhow::Result;
use cranelift::prelude::*;
use cranelift_codegen::ir::{Block, FuncRef, Function, JumpTable, StackSlot};
use std::collections::BTreeMap;

pub use {
    exit::Exit,
    register::{offsets, Registers},
};

mod control;
mod exit;
pub mod ir;
mod masm;
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
    pub blocks: BTreeMap<u64, Block>,

    /// The host call function
    pub host: BTreeMap<String, FuncRef>,

    /// The constants pool
    pub pool: Registers,

    /// Jump table for dynamic jumps
    pub jump: Vec<u64>,

    /// Stack slot for dynamic jumps
    pub stack: StackSlot,

    /// The runtime jump table
    pub rt_jump_table: JumpTable,

    /// The macro blocks
    pub masm: MacroBlocks,

    /// The memory info
    #[cfg(target_os = "macos")]
    pub memory: pvm::MemoryInfo,
}

impl<'b> Translator<'b> {
    /// Create a new translator with PVM register variables and PC
    pub fn new(
        blocks: &[u64],
        func: &'b mut Function,
        ctx: &'b mut FunctionBuilderContext,
    ) -> Result<Self> {
        let mut iblocks = BTreeMap::new();
        let mut builder = FunctionBuilder::new(func, ctx);
        for pc in blocks {
            iblocks.insert(*pc, builder.create_block());
        }

        Ok(Self {
            blocks: iblocks,
            host: BTreeMap::new(),
            jump: Vec::new(),
            pool: Registers::default(),
            stack: StackSlot::from_u32(0),
            rt_jump_table: JumpTable::new(0),
            masm: MacroBlocks::new(&mut builder),
            builder,
            #[cfg(target_os = "macos")]
            memory: pvm::MemoryInfo::default(),
        })
    }

    /// Reset the translator for new functions
    pub fn reset(
        &mut self,
        blocks: &[u64],
        func: &'b mut Function,
        ctx: &'b mut FunctionBuilderContext,
    ) {
        let mut iblocks = BTreeMap::new();
        let mut builder = FunctionBuilder::new(func, ctx);
        for pc in blocks {
            iblocks.insert(*pc, builder.create_block());
        }

        self.blocks = iblocks;
        self.masm = MacroBlocks::new(&mut builder);
        self.pool = Registers::default();
        self.builder = builder;
    }
}
