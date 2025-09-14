//! JIT module

use crate::{
    Engine, Memory, host,
    module::{self, ModuleLike},
    trap,
};
use anyhow::Result;
use cranelift_module::FuncId;
use pvm::{Argument, Program, Reason};

/// JIT module
pub struct JITModule(cranelift_jit::JITModule);

impl ModuleLike for JITModule {
    fn new<X: Argument>() -> Result<Self> {
        let mut builder = Engine::compilation()?;
        host::symbols::<X>(&mut builder);
        let module = cranelift_jit::JITModule::new(builder);
        Ok(Self(module))
    }

    fn compile(mut self, program: &Program) -> Result<Self> {
        module::compile(&mut self.0, program)?;
        self.0.finalize_definitions()?;
        Ok(self)
    }

    fn execute<X: Argument>(
        &self,
        ctx: &mut pvm::Context<'_, X, Memory>,
        pc: u64,
    ) -> Result<Reason> {
        let main = FuncId::from_u32(0);
        let func = unsafe {
            std::mem::transmute::<*const u8, module::MainSig<X>>(
                self.0.get_finalized_function(main),
            )
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

unsafe impl Send for JITModule {}
unsafe impl Sync for JITModule {}
