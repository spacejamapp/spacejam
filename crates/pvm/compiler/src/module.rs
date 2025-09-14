//! Compiled function metadata

use crate::{Memory, trap};
use anyhow::Result;
use cranelift_jit::JITModule;
use cranelift_module::FuncId;
use pvm::{Argument, Reason};

/// The signature of the main function
type MainSig<X> = fn(*mut pvm::Context<'_, X, Memory>, u64) -> (i64, i64);

/// Module with compiled code
pub struct Module {
    /// Code of the module
    pub jit: JITModule,
}

unsafe impl Send for Module {}
unsafe impl Sync for Module {}

impl Module {
    /// Execute compiled function
    pub fn execute<X: Argument>(
        &self,
        ctx: &mut pvm::Context<'_, X, Memory>,
        pc: u64,
    ) -> Result<Reason> {
        let main = FuncId::from_u32(0);
        let func = unsafe {
            std::mem::transmute::<*const u8, MainSig<X>>(self.jit.get_finalized_function(main))
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
