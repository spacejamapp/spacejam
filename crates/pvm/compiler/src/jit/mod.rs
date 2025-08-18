//! Clean block-based JIT compiler for PVM programs

use crate::module::Info;
use anyhow::Result;
use cranelift::prelude::*;
use cranelift_codegen::ir::{Function, UserFuncName};
use translator::{Code, Translator};
pub use {
    context::{Context, ExtendedContext},
    result::ExecResult,
};

mod context;
mod result;

/// JIT compiler
pub struct Jit {
    /// Cranelift context
    ctx: cranelift_codegen::Context,
    /// Cranelift ISA
    isa: cranelift_codegen::isa::OwnedTargetIsa,
}

impl Jit {
    /// Create new JIT compiler
    pub fn new() -> Result<Self> {
        let mut flag_builder = settings::builder();
        flag_builder
            .set("use_colocated_libcalls", "false")
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        flag_builder
            .set("is_pic", "false")
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let isa_builder = cranelift_native::builder().map_err(|e| anyhow::anyhow!("{}", e))?;
        let isa = isa_builder.finish(settings::Flags::new(flag_builder))?;

        Ok(Self {
            ctx: cranelift_codegen::Context::new(),
            isa,
        })
    }

    /// Execute program using JIT compilation
    pub fn execute(&mut self, program: &[u8], mut ctx: Context) -> Result<(Info, bool)> {
        tracing::debug!("Starting execution with initial PC: {}", ctx.pc);
        let (code, is_trap) = self.compile(program)?;
        self.run(&code, &mut ctx)?;

        Ok((
            Info {
                registers: ctx.registers,
                pc: ctx.pc,
                memory: ctx.memory.clone(),
            },
            is_trap,
        ))
    }

    /// Compile entire program as Cranelift function (cranelift-wasm style)
    fn compile(&mut self, program: &[u8]) -> Result<(Code, bool)> {
        tracing::debug!("Compiling entire program as Cranelift function");

        let mut sig = Signature::new(self.isa.default_call_conv());
        sig.params.push(AbiParam::new(types::I64)); // context pointer
        sig.params.push(AbiParam::new(types::I64)); // starting PC
        let mut func = Function::with_name_signature(UserFuncName::user(0, 0), sig);
        let mut builder_ctx = FunctionBuilderContext::new();
        let mut translator = Translator::new(&mut func, &mut builder_ctx)?;

        let is_trap = translator.analyze(program)?;

        // Create entry block
        let entry = translator.builder.create_block();
        translator
            .builder
            .append_block_params_for_function_params(entry);
        translator.builder.switch_to_block(entry);
        translator.translate(entry)?;

        // Finalize the function
        translator.builder.finalize();

        self.ctx.clear();
        self.ctx.func = func;
        let mut ctrl = cranelift_codegen::control::ControlPlane::default();
        self.ctx
            .compile(&*self.isa, &mut ctrl)
            .map_err(|e| anyhow::anyhow!("compilation failed: {:?}", e))?;

        let code = self.ctx.compiled_code().unwrap();
        let bytes = code.buffer.data();
        let size = bytes.len();

        let ptr = self.alloc_exec(size)?;
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, size);
        }

        tracing::debug!("compilation completed, generated {} bytes", size);
        Ok((Code { ptr, size }, is_trap))
    }

    /// Allocate executable memory
    fn alloc_exec(&self, size: usize) -> Result<*mut u8> {
        unsafe {
            let ptr = libc::mmap(
                std::ptr::null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            );

            if ptr == libc::MAP_FAILED {
                anyhow::bail!("Failed to allocate executable memory");
            }

            Ok(ptr as *mut u8)
        }
    }

    /// Execute compiled function
    fn run(&self, code: &Code, ctx: &mut Context) -> Result<()> {
        // Generate page allocation information for boundary checking
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

        unsafe {
            // Call the function with starting PC
            // It will execute the entire program from that PC
            let func = std::mem::transmute::<*const u8, fn(*mut ExtendedContext, u64)>(code.ptr);
            func(&mut ext_ctx, ctx.pc);
        }

        // Extract final state from the context
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
