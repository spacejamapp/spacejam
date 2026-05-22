//! Compiled function metadata

use crate::{Artifact, host, trap};
use anyhow::Result;
use cranelift::{
    codegen::{Context, control::ControlPlane, ir::Function, isa::CallConv},
    module::{self, Linkage, ModuleReloc},
    prelude::{AbiParam, FunctionBuilderContext, Signature, types},
};
use pvm::{Argument, Program, Reason};
use translator::Translator;
pub use {jit::JITModule, object::ObjectModule};

mod jit;
mod object;

/// The signature of the main function
pub type MainSig<X> = fn(*mut pvm::Context<'_, X, crate::Memory>, u64, i64) -> (i64, i64);

/// The name of the main function
pub const MAIN: &str = "main";

/// A trait for module-like objects.
pub trait ModuleLike: Sized {
    /// Create a new module
    fn new<X: Argument>() -> Result<Self>;

    /// Compile a program
    fn compile(self, program: &Program) -> Result<Self>;

    /// Try to load a previously-saved artifact for `program` without
    /// running codegen.
    fn try_load(self, _program: &Program) -> Result<Option<Self>> {
        Ok(None)
    }

    /// Get the main function
    fn main<X: Argument>(&self) -> Result<MainSig<X>>;

    /// Execute a program
    fn execute<X: Argument>(
        &self,
        ctx: &mut pvm::Context<'_, X, crate::Memory>,
        pc: u64,
    ) -> Result<Reason> {
        let main_fn = self.main::<X>()?;
        let result = match trap::with(|| main_fn(ctx, pc, host::table::<X>())) {
            Ok((gas, code)) => {
                let reason = translator::Exit::to_reason(code);
                tracing::debug!("exit code: {code}, reason: {reason:?}");
                ctx.set_gas(gas as u64);
                reason
            }
            Err(info) => Reason::Fault {
                page: info.address as u32 / pvm::PAGE_SIZE as u32,
            },
        };

        Ok(result)
    }
}

/// Declare functions for the program
pub fn compile(module: &mut impl module::Module, program: &Program) -> Result<()> {
    let signature = Signature {
        params: vec![AbiParam::new(types::I64); 3],
        returns: vec![AbiParam::new(types::I64); 2],
        call_conv: CallConv::Fast,
    };
    let mut ctx = module.make_context();
    let main = {
        let main = module.declare_function(MAIN, Linkage::Export, &signature)?;
        ctx.func.signature = signature.clone();
        main
    };

    // compile the program with cache
    let func = translate(&mut ctx, program)?;
    let isa = module.isa();
    let mut cpanel = ControlPlane::default();
    let (compiled, _hits) = ctx
        .compile_with_cache(isa, &mut Artifact, &mut cpanel)
        .map_err(|e| anyhow::anyhow!("failed to compile program: {:?}", e))?;

    // relocate the function
    let relocs = compiled
        .buffer
        .relocs()
        .iter()
        .map(|r| ModuleReloc::from_mach_reloc(r, &func, main))
        .collect::<Vec<_>>();

    module.define_function_bytes(main, 1, compiled.code_buffer(), &relocs)?;
    Ok(())
}

/// Translate the program to CLIF
fn translate(ctx: &mut Context, program: &Program) -> Result<Function> {
    let blob = program.blob()?;
    let code = blob.read_blocks()?;
    let minfo = program.memory.info.clone();
    let mut bctx = FunctionBuilderContext::new();
    let mut translator = Translator::new(&[], &mut ctx.func, &mut bctx)?;
    translator.jump = blob.jump_table.clone();
    translator.translate(program.registers, code, minfo.clone())?;
    Ok(ctx.func.clone())
}
