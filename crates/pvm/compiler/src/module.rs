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
                    // pc
                    *const u64,
                    // gas
                    *mut i64,
                    // memory
                    *mut Memory,
                    // registers
                    *mut u64,
                    *mut u64,
                    *mut u64,
                    *mut u64,
                    *mut u64,
                    *mut u64,
                    *mut u64,
                    *mut u64,
                    *mut u64,
                    *mut u64,
                    *mut u64,
                    *mut u64,
                    *mut u64,
                ) -> i64,
            >(self.code)
        };
        ctx.dispatch = self.dispatch;
        ctx.registers = self.registers;
        let result = match trap::with(|| {
            func(
                &pc,
                &mut ctx.gas,
                &mut ctx.memory,
                &mut ctx.registers[0],
                &mut ctx.registers[1],
                &mut ctx.registers[2],
                &mut ctx.registers[3],
                &mut ctx.registers[4],
                &mut ctx.registers[5],
                &mut ctx.registers[6],
                &mut ctx.registers[7],
                &mut ctx.registers[8],
                &mut ctx.registers[9],
                &mut ctx.registers[10],
                &mut ctx.registers[11],
                &mut ctx.registers[12],
            )
        }) {
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
