//! Compiler module that manages the compilation process

use crate::{jit::JitCompiler, Module};
use anyhow::Result;
use tracing;

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
    pub fn compile(&mut self, program_blob: &[u8]) -> Result<Module> {
        tracing::debug!("Compiling PVM program blob of {} bytes", program_blob.len());

        // Compile using JIT compiler
        self.jit.compile(program_blob)
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new().expect("Failed to create default compiler")
    }
}
