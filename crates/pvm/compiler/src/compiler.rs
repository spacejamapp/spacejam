//! Clean block-based JIT compiler for PVM programs

use crate::{Module, JIT};
use anyhow::Result;
use pvm::Program;

/// JIT compiler
pub struct Compiler {}

impl Compiler {
    /// Create new JIT compiler
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    /// Compile entire program as a function
    pub fn compile(&mut self, program: &Program) -> Result<Module> {
        JIT::new()?.compile(program)
    }
}
