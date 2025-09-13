//! Compiled function metadata

use crate::{Memory, trap};
use anyhow::Result;
use cranelift_jit::JITModule;
use cranelift_module::FuncId;
use pvm::{Argument, Reason};

/// Module with compiled code
pub struct Module {
    /// Code of the module
    pub jit: JITModule,
    /// The main function of the module
    pub main: FuncId,
    /// The registers for this module
    pub registers: [u64; pvm::REGISTER_COUNT],
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
        let func = unsafe {
            std::mem::transmute::<*const u8, fn(*mut pvm::Context<'_, X, Memory>, u64) -> (i64, i64)>(
                self.jit.get_finalized_function(self.main),
            )
        };
        ctx.registers = self.registers;
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

    /// Invoke the compiled module
    ///
    /// NOTE: this API is mainly used for testing
    pub fn invoke(
        &self,
        registers: &[u64; pvm::REGISTER_COUNT],
        pc: u64,
        gas: u64,
        memory: pvm::Memory,
    ) -> Result<Info> {
        let mut context = pvm::Context {
            registers: *registers,
            gas: gas as i64,
            memory: Memory::new(&memory).expect("failed to create memory"),
            ctx: &mut (),
        };

        let reason = self.execute(&mut context, pc)?;
        Ok(Info {
            registers: context.registers,
            gas: context.gas as u64,
            memory: context.memory.fill(&memory),
            reason,
        })
    }
}

/// Result of executing a compiled module
#[derive(Debug, Clone)]
pub struct Info {
    /// Final register values
    pub registers: [u64; pvm::REGISTER_COUNT],

    /// Final gas
    pub gas: u64,

    /// Final memory state
    pub memory: pvm::Memory,

    /// The exit reason
    pub reason: Reason,
}
