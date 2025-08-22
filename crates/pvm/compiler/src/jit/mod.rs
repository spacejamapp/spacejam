//! Cranelift JIT backend

use anyhow::Result;
use cranelift_jit::JITBuilder;

/// Cranelift JIT module builder
pub struct Jit {
    /// Cranelift JIT module builder
    pub builder: JITBuilder,
}

impl Jit {
    /// Create new JIT module builder
    pub fn new() -> Result<Self> {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names())?;
        Ok(Self { builder })
    }
}
