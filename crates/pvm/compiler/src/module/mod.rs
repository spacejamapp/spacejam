//! Compiled function metadata

use crate::{trap, Memory};
use anyhow::Result;
use pvm::Reason;
pub use {context::Context, info::Info};

mod context;
mod info;

/// Module with compiled code
pub struct Module {
    /// The function composed by cranelift IR
    code: *const u8,
    /// The virtual memory for this module
    memory: Memory,
}

impl Module {
    /// Set the program bytes for block JIT execution with memory
    pub fn new(code: *const u8, memory: Memory) -> Self {
        Self { code, memory }
    }

    /// Execute the compiled module
    pub fn execute(
        &self,
        initial_registers: &[u64; pvm::REGISTER_COUNT],
        initial_pc: u64,
        initial_gas: u64,
        initial_memory: pvm::Memory,
    ) -> Result<Info> {
        let mut context = Context::new(*initial_registers, initial_pc, self.memory.clone());
        let reason = self.run(&mut context)?;
        Ok(Info {
            registers: context.registers,
            pc: context.pc,
            gas: initial_gas.saturating_sub(context.gas),
            memory: context.memory.fill(&initial_memory),
            reason,
        })
    }

    /// Execute compiled function
    fn run(&self, ctx: &mut Context) -> Result<Reason> {
        let func = unsafe { std::mem::transmute::<*const u8, fn(*mut Context) -> u8>(self.code) };
        let result = match trap::with(|| func(ctx)) {
            Ok(r) => match r {
                0 => Reason::Halt,
                1 => Reason::Panic("Trap".to_string()),
                4 => Reason::OOG,
                _ => Reason::Panic("Unknown exit code".to_string()),
            },
            Err(info) => Reason::Fault {
                page: info.address as u32 / pvm::PAGE_SIZE as u32,
            },
        };

        Ok(result)
    }
}
