//! Clean block-based JIT compiler for PVM programs

use crate::{Module, JIT};
use anyhow::Result;

/// JIT compiler
pub struct Compiler {}

impl Compiler {
    /// Create new JIT compiler
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    /// Compile entire program as a function
    pub fn compile(&mut self, program: &[u8]) -> Result<Module> {
        JIT::new()?.compile(program)
    }
}
