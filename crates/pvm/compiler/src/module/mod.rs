//! Compiled function metadata

use crate::{trap, Memory};
use anyhow::Result;
pub use info::Info;
use pvm::{Argument, Reason};

mod info;

/// Module with compiled code
pub struct Module {
    /// The function composed by cranelift IR
    pub code: *const u8,
    /// The virtual memory for this module
    pub memory: Memory,
    /// The registers for this module
    pub registers: [u64; pvm::REGISTER_COUNT],
}

impl Module {
    /// Invoke the compiled module
    pub fn invoke(
        &self,
        registers: &[u64; pvm::REGISTER_COUNT],
        pc: u64,
        gas: u64,
        memory: pvm::Memory,
    ) -> Result<Info> {
        let mut context = pvm::Context {
            registers: *registers,
            pc,
            gas: gas as i64,
            memory: self.memory.clone(),
            ctx: &mut (),
        };

        let reason = self.run(&mut context)?;
        Ok(Info {
            registers: context.registers,
            pc: context.pc,
            gas: context.gas as u64,
            memory: context.memory.fill(&memory),
            reason,
        })
    }

    /// Execute the module with given context
    pub fn execute<X: Argument>(&self, _ctx: &mut pvm::Context<'_, (), Memory>) -> Result<Reason> {
        todo!()
    }

    /// Execute compiled function
    fn run(&self, ctx: &mut pvm::Context<'_, (), Memory>) -> Result<Reason> {
        let func = unsafe {
            std::mem::transmute::<*const u8, fn(*mut pvm::Context<'_, (), Memory>) -> u8>(self.code)
        };
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
