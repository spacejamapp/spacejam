//! Compiled function metadata

use anyhow::Result;
pub use {
    context::{Context, ExtendedContext},
    info::{ExecResult, Info},
};

mod context;
mod info;

/// Module with compiled code
#[derive(Debug, Clone)]
pub struct Module {
    /// The function composed by cranelift IR
    code: *const u8,

    /// Whether the program is a trap
    ///
    /// FIXME: this is currently a workaround for tests
    is_trap: bool,
}

impl Module {
    /// Set the program bytes for block JIT execution
    pub fn new(code: *const u8, is_trap: bool) -> Self {
        Self { code, is_trap }
    }

    /// Execute the compiled module
    pub fn execute(
        &self,
        initial_registers: &[u64; pvm::REGISTER_COUNT],
        initial_pc: u64,
        initial_memory: pvm::Memory,
    ) -> Result<Info> {
        let mut context = Context::new(*initial_registers, initial_pc, initial_memory);
        self.run(&mut context)?;
        let final_pc = if initial_pc == 0 && context.pc == 1 && self.is_trap {
            0
        } else {
            context.pc
        };

        Ok(Info {
            registers: context.registers,
            pc: final_pc,
            memory: context.memory,
        })
    }

    /// Execute compiled function
    fn run(&self, ctx: &mut Context) -> Result<()> {
        let (page_bitmap, page_access) = ctx.generate_page_bitmap();
        let mut ext_ctx = ExtendedContext {
            registers: ctx.registers,
            pc: ctx.pc,
            memory_ptr: ctx.linear_mem.as_mut_ptr(),
            page_bitmap: page_bitmap.as_ptr(),
            page_access: page_access.as_ptr(),
            result: ExecResult::Continue,
            pc_managed: false,
        };

        let func =
            unsafe { std::mem::transmute::<*const u8, fn(*mut ExtendedContext, u64)>(self.code) };
        func(&mut ext_ctx, ctx.pc);
        ctx.registers = ext_ctx.registers;
        ctx.pc = ext_ctx.pc;

        // Check for page faults
        match ctx.sync() {
            Ok(_) => {
                tracing::trace!("execution completed, final PC: {}", ctx.pc);
                Ok(())
            }
            Err(e) => {
                tracing::trace!("Page fault detected during execution: {}", e);
                ctx.pc = 0;
                Ok(())
            }
        }
    }
}
