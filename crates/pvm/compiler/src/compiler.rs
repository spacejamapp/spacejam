//! Clean block-based JIT compiler for PVM programs

use crate::Module;
use anyhow::Result;
use cranelift::prelude::*;
use cranelift_codegen::ir::{Function, UserFuncName};
use translator::Translator;

/// JIT compiler
pub struct Compiler {
    /// Cranelift ISA
    isa: cranelift_codegen::isa::OwnedTargetIsa,
}

impl Compiler {
    /// Create new JIT compiler
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

        Ok(Self { isa })
    }

    /// Compile entire program as a function
    pub fn compile(&mut self, program: &[u8]) -> Result<Module> {
        tracing::debug!("Compiling entire program as Cranelift function");

        let mut sig = Signature::new(self.isa.default_call_conv());
        sig.params.push(AbiParam::new(types::I64)); // context pointer
        sig.params.push(AbiParam::new(types::I64)); // starting PC
        let mut func = Function::with_name_signature(UserFuncName::user(0, 0), sig);
        let mut builder_ctx = FunctionBuilderContext::new();
        let mut translator = Translator::new(&mut func, &mut builder_ctx)?;
        let is_trap = translator.analyze(program)?;

        // Translate the program
        translator.translate()?;
        translator.builder.finalize();

        let mut ctx = cranelift_codegen::Context::new();
        ctx.func = func;
        let mut ctrl = cranelift_codegen::control::ControlPlane::default();
        ctx.compile(&*self.isa, &mut ctrl)
            .map_err(|e| anyhow::anyhow!("compilation failed: {:?}", e))?;

        let code = ctx.compiled_code().unwrap();
        Ok(Module::new(code.clone(), is_trap))
    }
}
