//! Compiled function metadata

use crate::{trap, Memory};
use anyhow::Result;
use pvm::{Argument, Reason};

/// Module with compiled code
pub struct Module {
    /// The function composed by cranelift IR
    pub code: *const u8,
    /// The virtual memory for this module
    pub memory: Memory,
    /// The registers for this module
    pub registers: [u64; pvm::REGISTER_COUNT],
    /// The function table for this module
    pub dispatch: [u64; pvm::MAX_FUNCTIONS],
}

impl Module {
    /// Execute compiled function
    pub fn execute<X: Argument>(&self, ctx: &mut pvm::Context<'_, X, Memory>) -> Result<Reason> {
        let func = unsafe {
            std::mem::transmute::<*const u8, fn(*mut pvm::Context<'_, X, Memory>) -> i64>(self.code)
        };
        ctx.dispatch = self.dispatch;
        let result = match trap::with(|| func(ctx)) {
            Ok(r) => {
                let reason = translator::Exit::to_reason(r);
                tracing::debug!("exit code: {r}, reason: {reason:?}");
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
        _pc: u64,
        gas: u64,
        memory: pvm::Memory,
    ) -> Result<Info> {
        let mut context = pvm::Context {
            dispatch: self.dispatch,
            registers: *registers,
            gas: gas as i64,
            memory: self.memory.clone(),
            ctx: &mut (),
        };

        let reason = self.execute(&mut context)?;
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
