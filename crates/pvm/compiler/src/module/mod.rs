//! Compiled function metadata

use crate::{Artifact, Memory, engine, host, trap};
use anyhow::Result;
use cranelift::prelude::{AbiParam, FunctionBuilderContext, Signature, types};
use cranelift_codegen::{Context, control::ControlPlane, ir::Function, isa::CallConv};
use cranelift_jit::JITModule;
use cranelift_module::FuncId;
use cranelift_module::{Linkage, Module as _, ModuleReloc};
use pvm::{Argument, Program, Reason};
use translator::Translator;

/// The signature of the main function
type MainSig<X> = fn(*mut pvm::Context<'_, X, Memory>, u64) -> (i64, i64);

/// Module with compiled code
pub struct Module {
    /// Code of the module
    pub module: JITModule,
}

const MAIN: &str = "main";

impl Module {
    /// Create new JIT module builder for host functions
    pub fn new<X: Argument>() -> Result<Self> {
        let mut builder = engine::compilation()?;
        host::symbols::<X>(&mut builder);
        let module = JITModule::new(builder);
        Ok(Self { module })
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
        let mut artifact = Artifact::new()?;
        let func = self.translate(&mut ctx, program)?;
        let isa = self.module.isa();
        let mut cpanel = ControlPlane::default();
        let (compiled, _hits) = ctx
            .compile_with_cache(isa, &mut artifact, &mut cpanel)
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
        Ok(self)
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

    /// Execute compiled function
    pub fn execute<X: Argument>(
        &self,
        ctx: &mut pvm::Context<'_, X, Memory>,
        pc: u64,
    ) -> Result<Reason> {
        let main = FuncId::from_u32(0);
        let func = unsafe {
            std::mem::transmute::<*const u8, MainSig<X>>(self.module.get_finalized_function(main))
        };
        let result = match trap::with(|| func(ctx, pc)) {
            Ok((gas, code)) => {
                let reason = translator::Exit::to_reason(code);
                tracing::debug!("exit code: {code}, reason: {reason:?}");
                ctx.gas = gas;
                reason
            }
            Err(info) => Reason::Fault {
                page: info.address as u32 / pvm::PAGE_SIZE as u32,
            },
        };
        Ok(result)
    }
}

unsafe impl Send for Module {}
unsafe impl Sync for Module {}
