//! Cranelift JIT backend

use crate::{Compiler, Module};
use anyhow::Result;
use cranelift::prelude::{AbiParam, FunctionBuilderContext, Signature, types};
use cranelift_codegen::{Context, control::ControlPlane, ir::Function, isa::CallConv};
use cranelift_module::{Linkage, Module as _, ModuleReloc};
use pvm::Program;
use translator::Translator;

const MAIN: &str = "main";

impl Compiler {
    /// Compile the program with cache
    pub fn compile_with_cache(self, program: &Program) -> Result<Module> {
        self.compile(program)
    }

    /// Declare functions for the program
    pub fn compile(mut self, program: &Program) -> Result<Module> {
        let signature = Signature {
            params: vec![AbiParam::new(types::I64); 2],
            returns: vec![AbiParam::new(types::I64); 2],
            call_conv: CallConv::Fast,
        };
        let mut ctx = self.module.make_context();
        let main = {
            let main = self
                .module
                .declare_function(MAIN, Linkage::Export, &signature)?;
            ctx.func.signature = signature.clone();
            main
        };

        // compile the program with cache
        let func = self.translate(&mut ctx, program)?;
        let isa = self.module.isa();
        let mut cpanel = ControlPlane::default();
        let (compiled, _hits) = ctx
            .compile_with_cache(isa, &mut self.artifact, &mut cpanel)
            .map_err(|e| anyhow::anyhow!("failed to compile program: {:?}", e))?;

        // relocate the function
        let relocs = compiled
            .buffer
            .relocs()
            .iter()
            .map(|r| ModuleReloc::from_mach_reloc(r, &func, main))
            .collect::<Vec<_>>();

        self.module
            .define_function_bytes(main, 1, compiled.code_buffer(), &relocs)?;
        self.module.finalize_definitions()?;
        compiled.buffer.data();

        Ok(Module {
            jit: self.module,
            main,
        })
    }

    /// Translate the program to CLIF
    fn translate(&mut self, ctx: &mut Context, program: &Program) -> Result<Function> {
        let host = self.declare_host_in_module()?;
        let blob = program.blob()?;
        let code = blob.read_blocks()?;
        let host = self.declare_host_in_func(host, &mut ctx.func)?;
        let minfo = program.memory.info.clone();
        let mut bctx = FunctionBuilderContext::new();
        let mut translator = Translator::new(&[], &mut ctx.func, &mut bctx)?;
        translator.jump = blob.jump_table.clone();
        translator.host = host;
        translator.translate(program.registers, code, minfo.clone())?;
        if std::env::var("DUMP_CLIF").is_ok() {
            println!("{}", &ctx.func);
        }
        Ok(ctx.func.clone())
    }
}
