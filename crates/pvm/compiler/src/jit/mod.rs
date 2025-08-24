//! Cranelift JIT backend

use anyhow::Result;
use cranelift::prelude::{types, AbiParam, FunctionBuilderContext, Signature};
use cranelift_codegen::{ir::GlobalValue, Context};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataDescription, Linkage, Module};
use pvm::Program;
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
    pub fn compile(&mut self, program: &Program) -> Result<crate::Module> {
        let sig = self.signature();
        let id = self
            .module
            .declare_function("main", Linkage::Export, &sig)?;

        // construct the function
        self.ctx.func.signature = self.signature();
        let ro_data = self.make_data("ro_data", &program.memory.ro_data()?, false)?;
        let rw_data = self.make_data("rw_data", &program.memory.rw_data()?, true)?;
        let mut trans = Translator::new(&mut self.ctx.func, &mut self.bctx)?.data(ro_data, rw_data);

        // translate the function
        trans.translate(program)?;
        trans.builder.finalize();

        // define the function
        self.module.define_function(id, &mut self.ctx)?;
        self.module.clear_context(&mut self.ctx);
        self.module.finalize_definitions()?;
        Ok(crate::Module::new(self.module.get_finalized_function(id)))
    }

    /// Create data for the function
    fn make_data(&mut self, name: &str, data: &[u8], writable: bool) -> Result<GlobalValue> {
        let mut desc = DataDescription::new();
        let id = self
            .module
            .declare_data(name, Linkage::Local, writable, false)?;
        desc.define(data.into());
        self.module.define_data(id, &desc)?;
        Ok(self.module.declare_data_in_func(id, &mut self.ctx.func))
    }

    /// Create a signature for the function
    fn signature(&self) -> Signature {
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(types::I64));
        sig.returns.push(AbiParam::new(types::I8));
        sig
    }
}
