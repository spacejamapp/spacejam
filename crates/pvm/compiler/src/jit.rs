//! Cranelift JIT backend

use crate::host;
use anyhow::Result;
use cranelift::prelude::{types, AbiParam, FunctionBuilderContext, Signature};
use cranelift_codegen::Context;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use pvm::{Argument, Program};
use translator::Translator;

const MAIN: &str = "main";

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

    /// Create new JIT module builder for host functions
    pub fn host<X: Argument>() -> Result<Self> {
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names())?;
        builder.symbol(host::CALL, host::call::<X> as *const u8);
        builder.symbol(host::SBRK, host::sbrk::<X> as *const u8);
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
    pub fn compile(&mut self, program: &Program) -> Result<crate::Module> {
        let memory = crate::Memory::new(&program.memory)?;
        let sig = self.signature();
        let id = self.module.declare_function(MAIN, Linkage::Export, &sig)?;

        // construct the function
        let host = self.declare_host()?;
        self.ctx.func.signature = self.signature();
        let mut trans = Translator::new(&mut self.ctx.func, &mut self.bctx)?;
        trans.host = host;
        trans.translate(program)?;
        trans.builder.finalize();

        // define the function
        self.module.define_function(id, &mut self.ctx)?;
        self.module.clear_context(&mut self.ctx);
        self.module.finalize_definitions()?;
        Ok(crate::Module {
            code: self.module.get_finalized_function(id),
            memory,
            registers: program.registers,
        })
    }

    /// Create a signature for the function
    fn signature(&self) -> Signature {
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(types::I64));
        sig.returns.push(AbiParam::new(types::I8));
        sig
    }
}
