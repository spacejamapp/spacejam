//! SpaceJam PVM compiler

use crate::Module;
use anyhow::Result;
use cranelift::prelude::*;
use cranelift_codegen::settings;

/// SpaceJam PVM compiler
pub struct Compiler {
    /// Cranelift context
    _ctx: cranelift_codegen::Context,

    /// Cranelift ISA
    _isa: cranelift_codegen::isa::OwnedTargetIsa,
}

impl Compiler {
    /// Create new compiler
    pub fn new() -> Result<Self> {
        let mut flag_builder = settings::builder();
        flag_builder
            .set("use_colocated_libcalls", "false")
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        flag_builder
            .set("is_pic", "false")
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let isa_builder = cranelift_native::builder().map_err(|e| anyhow::anyhow!("{}", e))?;
        let isa = isa_builder.finish(settings::Flags::new(flag_builder))?;

        Ok(Self {
            _ctx: cranelift_codegen::Context::new(),
            _isa: isa,
        })
    }

    /// Compile a PVM program
    ///
    /// TODO: we need to split the program into refine and accumulate in the future
    pub fn compile(&self, _program: &[u8]) -> Result<Module> {
        todo!()
    }

    /// Compile a single function
    pub fn compile_function(&self, _program: &[u8]) -> Result<Module> {
        todo!()
    }
}
