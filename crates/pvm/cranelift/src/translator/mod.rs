//! Translator module that converts PVM instructions to Cranelift IR

use anyhow::Result;
use cranelift::prelude::*;
use cranelift_codegen::ir;
use std::collections::HashMap;

mod context;
mod control;
mod memory;
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

    /// Map of PVM blocks by start PC
    pvm_blocks: HashMap<u64, Block>,

    // Context pointer for boundary checking and runtime operations
    ctx_ptr: Option<Value>,
}

impl<'b> Translator<'b> {
    /// Create a new translator with PVM register variables and PC
    pub fn new(func: &'b mut ir::Function, ctx: &'b mut FunctionBuilderContext) -> Result<Self> {
        let mut builder = FunctionBuilder::new(func, ctx);

        // Declare all PVM registers as Cranelift variables
        // PVM has registers: ra(0), sp(1), unused(2,3,4), s0-s1(5-6), a0-a4(7-11), unused(12)
        let mut registers = HashMap::new();
        for i in 0..pvm::REGISTER_COUNT {
            let var = Variable::new(i);
            builder.declare_var(var, types::I64);
            registers.insert(i as u8, var);
        }

        Ok(Self {
            registers,
            builder,
            jump_table: Vec::new(),
            blocks: HashMap::new(),
            ctx_ptr: None,
            pvm_blocks: HashMap::new(),
        })
    }

    /// Initialize blocks
    pub fn init_blocks(&mut self, blocks: HashMap<u64, ir::Block>) {
        self.blocks = blocks;
    }

    /// Initialize translator with runtime context (context loading handled by runtime)
    pub fn init_with_context(&mut self, context_ptr: Value) -> Result<()> {
        self.ctx_ptr = Some(context_ptr);
        Ok(())
    }
}

/// Basic block - single entry/exit instruction sequence with pre-parsed instructions
#[derive(Debug, Clone)]
pub struct Block {
    /// Start PC of the block
    pub start: usize,
    /// End PC of the block
    pub end: usize,
    /// Whether the block terminates
    pub terminates: bool,
    /// Pre-parsed instructions for this block
    pub instructions: Vec<parser::reader::Offset<parser::Instruction>>,
}

/// Compiled native code
#[derive(Debug, Clone)]
pub struct Code {
    /// Pointer to the compiled code
    pub ptr: *const u8,

    /// Size of the compiled code
    pub size: usize,
}
