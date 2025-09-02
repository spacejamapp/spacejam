//! Cranelift JIT backend

use crate::{engine, host, Artifact};
use anyhow::Result;
use cranelift::prelude::{types, AbiParam, FunctionBuilderContext, Signature};
use cranelift_codegen::Context;
use cranelift_jit::JITModule;
use cranelift_module::{FuncId, Linkage, Module, ModuleReloc};
use pvm::{score::OpaqueHash, Argument, Program};
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

    /// Artifact
    pub artifact: Artifact,
}

impl JIT {
    /// Create new JIT module builder
    pub fn new() -> Result<Self> {
        let mut builder = engine::compilation()?;
        host::symbols::<pvm::Context<'_, (), crate::Memory>>(&mut builder);
        let module = JITModule::new(builder);
        Ok(Self {
            bctx: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            module,
            artifact: Artifact::new()?,
        })
    }

    /// Create new JIT module builder for host functions
    pub fn host<X: Argument>() -> Result<Self> {
        let mut builder = engine::compilation()?;
        host::symbols::<X>(&mut builder);
        let module = JITModule::new(builder);
        Ok(Self {
            bctx: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            module,
            artifact: Artifact::new()?,
        })
    }

    /// Compile a program
    ///
    /// TODO: introduce different artifacts for different pc, e.g.
    /// - accumulate
    /// - refine
    /// - is_authorized
    /// - core_vm ???
    ///
    /// FIXME: clean the API later then.
    pub fn compile(
        &mut self,
        program: &Program,
        hash: Option<OpaqueHash>,
    ) -> Result<crate::Module> {
        let memory = crate::Memory::new(&program.memory)?;
        let id = self.clif(program, hash)?;
        let func = self.ctx.func.clone();

        // define the function
        let (compiled, hits) = self
            .ctx
            .compile_with_cache(
                self.module.isa(),
                &mut self.artifact,
                &mut Default::default(),
            )
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;

        let relocs = compiled
            .buffer
            .relocs()
            .iter()
            .map(|r| ModuleReloc::from_mach_reloc(r, &func, id))
            .collect::<Vec<_>>();
        self.module
            .define_function_bytes(id, 1, compiled.code_buffer(), &relocs)?;
        self.module.finalize_definitions()?;
        if let Some(hash) = hash.and_then(|h| hits.then_some(h)) {
            self.artifact.put(hash, &func, true)?;
        }

        self.module.clear_context(&mut self.ctx);
        Ok(crate::Module {
            code: self.module.get_finalized_function(id),
            memory,
            registers: program.registers,
        })
    }

    /// Translate the program to CLIF
    fn clif(&mut self, program: &Program, hash: Option<OpaqueHash>) -> Result<FuncId> {
        let host = self.declare_host_in_module()?;
        if let Some((fun, _)) = hash.and_then(|h| self.artifact.clif(h)) {
            self.ctx = Context::for_function(fun);
            let fun =
                self.module
                    .declare_function(MAIN, Linkage::Export, &self.ctx.func.signature)?;
            return Ok(fun);
        }

        // construct the function
        let sig = self.signature();
        let id = self.module.declare_function(MAIN, Linkage::Export, &sig)?;
        let host = self.declare_host_in_func(host)?;
        self.ctx.func.signature = self.signature();
        let mut trans = Translator::new(&mut self.ctx.func, &mut self.bctx)?;
        trans.host = host;
        trans.translate(program)?;
        trans.builder.finalize();

        // cache the function
        if let Some(hash) = hash {
            self.artifact.put(hash, &self.ctx.func, false)?;
        }

        Ok(id)
    }

    /// Create a signature for the function
    fn signature(&self) -> Signature {
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(types::I64));
        sig.returns.push(AbiParam::new(types::I64));
        sig
    }
}
