//! Compiled function metadata

use anyhow::Result;
pub use {
    context::Context,
    info::{ExecResult, Info},
};

mod context;
mod info;

/// Module with compiled code
#[derive(Debug, Clone)]
pub struct Module {
    /// The function composed by cranelift IR
    code: *const u8,
}

impl Module {
    /// Set the program bytes for block JIT execution
    pub fn new(code: *const u8) -> Self {
        Self { code }
    }

    /// Execute the compiled module
    pub fn execute(
        &self,
        initial_registers: &[u64; pvm::REGISTER_COUNT],
        initial_pc: u64,
        initial_gas: u64,
        initial_memory: pvm::Memory,
    ) -> Result<Info> {
        let mut context = Context::new(*initial_registers, initial_pc, initial_memory);
        self.run(&mut context)?;
        Ok(Info {
            registers: context.registers,
            pc: context.pc,
            gas: initial_gas.saturating_sub(context.gas),
            memory: context.memory,
        })
    }

    /// Execute compiled function
    fn run(&self, ctx: &mut Context) -> Result<u8> {
        let mut ext = ctx.extend();
        let func = unsafe {
            std::mem::transmute::<*const u8, fn(*mut translator::Context) -> u8>(self.code)
        };
        let result = func(&mut ext);
        ctx.registers = ext.registers;
        ctx.pc = ext.pc;
        ctx.gas = ext.gas;
        tracing::debug!("result: {:?}", result);
        Ok(result)
    }
}
