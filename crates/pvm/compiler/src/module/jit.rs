//! JIT module

use crate::{
    Engine, host,
    module::{self, ModuleLike},
    trap,
};
use anyhow::Result;
use cranelift::{
    jit,
    jit::JITBuilder,
    module::{FuncId, default_libcall_names},
};
use pvm::{Argument, Program, Reason};

/// JIT module
pub struct JITModule(jit::JITModule);

impl ModuleLike for JITModule {
    fn new<X: Argument>() -> Result<Self> {
        let isa = Engine::compilation()?;
        let builder = JITBuilder::with_isa(isa, default_libcall_names());
        let module = jit::JITModule::new(builder);
        Ok(Self(module))
    }

    fn compile(mut self, program: &Program) -> Result<Self> {
        module::compile(&mut self.0, program)?;
        self.0.finalize_definitions()?;
        Ok(self)
    }

    fn execute<X: Argument>(
        &self,
        ctx: &mut pvm::Context<'_, X, crate::Memory>,
        pc: u64,
    ) -> Result<Reason> {
        let main = FuncId::from_u32(0);
        let func = unsafe {
            std::mem::transmute::<*const u8, module::MainSig<X>>(
                self.0.get_finalized_function(main),
            )
        };
        let result = match trap::with(|| {
            tracing::debug!(
                "JIT: About to call main function with ctx={:p}, pc={}, table={:#x}",
                ctx as *mut _,
                pc,
                host::table::<X>()
            );
            func(ctx, pc, host::table::<X>())
        }) {
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

unsafe impl Send for JITModule {}
unsafe impl Sync for JITModule {}
