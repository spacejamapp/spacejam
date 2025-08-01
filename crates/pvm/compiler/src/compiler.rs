//! Compiler module that manages the compilation process

use crate::{jit::JitCompiler, Module};
use anyhow::Result;

/// Main compiler struct that manages compilation state
pub struct Compiler {
    /// JIT compiler for code generation
    jit: JitCompiler,
}

impl Compiler {
    /// Create a new compiler instance
    pub fn new() -> Result<Self> {
        Ok(Self {
            jit: JitCompiler::new()?,
        })
    }

    /// Compile a PVM program blob to native code
    ///
    /// TODO: cache the compiled programs using hash as index.
    pub fn compile(&mut self, program_blob: &[u8], registers: [u64; 13]) -> Result<Module> {
        self.jit.compile(program_blob, registers)
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new().expect("Failed to create default compiler")
    }
}
