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
    pub fn execute<X: Argument>(
        &self,
        ctx: &mut pvm::Context<'_, X, Memory>,
        pc: u64,
    ) -> Result<Reason> {
        let func = unsafe {
            std::mem::transmute::<
                *const u8,
                fn(
                    // vmctx
                    *mut pvm::Context<'_, X, Memory>,
                    // pc
                    u8,
                    // gas
                    i64,
                    // registers
                    u64,
                    u64,
                    u64,
                    u64,
                    u64,
                    u64,
                    u64,
                    u64,
                    u64,
                    u64,
                    u64,
                    u64,
                    u64,
                ) -> (i64, i64),
            >(self.code)
        };
        ctx.dispatch = self.dispatch;
        ctx.registers = self.registers;
        let result = match trap::with(|| {
            func(
                ctx,
                pc as u8,
                ctx.gas.clone(),
                ctx.registers[0],
                ctx.registers[1],
                ctx.registers[2],
                ctx.registers[3],
                ctx.registers[4],
                ctx.registers[5],
                ctx.registers[6],
                ctx.registers[7],
                ctx.registers[8],
                ctx.registers[9],
                ctx.registers[10],
                ctx.registers[11],
                ctx.registers[12],
            )
        }) {
            Ok((gas, code)) => {
                let reason = translator::Exit::to_reason(code as i64);
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
            dispatch: self.dispatch,
            registers: *registers,
            gas: gas as i64,
            memory: self.memory.clone(),
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
