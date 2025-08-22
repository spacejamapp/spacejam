//! Cranelift JIT backend

use anyhow::Result;
use cranelift::prelude::{types, AbiParam, FunctionBuilderContext, Signature};
use cranelift_codegen::Context;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use translator::Translator;

/// Cranelift JIT module builder
pub struct JIT {
    /// Cranelift JIT module builder
    pub module: JITModule,

    /// Function builder context
    pub bctx: FunctionBuilderContext,

    /// Cranelift codegen context
    pub ctx: Context,
}

impl JIT {
    /// Create new JIT module builder
    pub fn new() -> Result<Self> {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names())?;
        let module = JITModule::new(builder);
        Ok(Self {
            bctx: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            module,
        })
    }

    /// Compile a program
    ///
    /// TODO: introduce different artifacts for different pc, e.g.
    /// - accumulate
    /// - refine
    /// - is_authorized
    /// - core_vm ???
    pub fn compile(&mut self, program: &[u8]) -> Result<crate::Module> {
        let sig = self.signature();
        let id = self
            .module
            .declare_function("main", Linkage::Export, &sig)?;

        let is_trap = self.translate(program)?;
        self.module.define_function(id, &mut self.ctx)?;
        self.module.clear_context(&mut self.ctx);
        self.module.finalize_definitions()?;
        Ok(crate::Module::new(
            self.module.get_finalized_function(id),
            is_trap,
        ))
    }

    fn translate(&mut self, program: &[u8]) -> Result<bool> {
        self.ctx.func.signature = self.signature();
        let mut trans = Translator::new(&mut self.ctx.func, &mut self.bctx)?;
        let is_trap = trans.translate(program)?;
        trans.builder.finalize();
        Ok(is_trap)
    }

    /// Create a signature for the function
    fn signature(&self) -> Signature {
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(types::I64)); // context pointer
        sig.params.push(AbiParam::new(types::I64)); // starting PC
        sig
    }
}
