//! JIT compiler that generates and executes native code using Cranelift

use crate::{Module, Translator};
use anyhow::Result;
use cranelift::prelude::*;
use cranelift_codegen::ir::{Function, UserFuncName};
use cranelift_codegen::Context;
use cranelift_native::builder;

/// JIT compiler that generates and executes native code using Cranelift
pub struct JitCompiler {
    /// Cranelift code generator context
    context: Context,
    /// Target ISA for code generation
    isa: cranelift_codegen::isa::OwnedTargetIsa,
}

impl JitCompiler {
    /// Create a new JIT compiler with proper Cranelift setup
    pub fn new() -> Result<Self> {
        // Create target ISA for the current platform
        let mut flag_builder = settings::builder();
        flag_builder
            .set("use_colocated_libcalls", "false")
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        flag_builder
            .set("is_pic", "false")
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let isa_builder = builder().map_err(|e| anyhow::anyhow!("{}", e))?;
        let isa = isa_builder.finish(settings::Flags::new(flag_builder))?;

        Ok(Self {
            context: Context::new(),
            isa,
        })
    }

    /// Compile PVM program to native code using Cranelift
    pub fn compile(&mut self, program: &[u8]) -> Result<Module> {
        // Clear previous compilation state
        self.context.clear();

        // Create function signature (takes *mut ExecutionContext param, no returns)
        let mut sig = Signature::new(self.isa.default_call_conv());
        sig.params.push(AbiParam::new(types::I64)); // pointer to ExecutionContext

        // Create function and builder
        let mut func = Function::with_name_signature(UserFuncName::user(0, 0), sig);
        let mut builder_context = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut func, &mut builder_context);

        // Create entry block
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        // Get the ExecutionContext pointer parameter
        let context_ptr = builder.block_params(entry_block)[0];

        // Create translator and load initial execution context
        let mut translator = Translator::new(&mut builder);
        translator.load_initial_context(context_ptr)?;

        let (registers, pc) = translator.translate(program)?;

        // Only add return handling for linear programs (no control flow)
        if !registers.is_empty() {
            // Store all 13 register values back to context.registers
            for (i, value) in registers.iter().enumerate() {
                let offset = builder.ins().iconst(types::I64, (i * 8) as i64);
                let addr = builder.ins().iadd(context_ptr, offset);
                builder.ins().store(MemFlags::new(), *value, addr, 0);
            }

            // Store PC back to context.pc (offset 104)
            let pc_offset = builder.ins().iconst(types::I64, 104);
            let pc_addr = builder.ins().iadd(context_ptr, pc_offset);
            builder.ins().store(MemFlags::new(), pc, pc_addr, 0);

            builder.ins().return_(&[]);
        }
        // For control flow programs, translate() already handled the return
        builder.finalize();

        // Compile the function
        self.context.func = func;
        let mut ctrl_plane = cranelift_codegen::control::ControlPlane::default();
        self.context
            .compile(&*self.isa, &mut ctrl_plane)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        // Get the compiled machine code
        let code = self.context.compiled_code().unwrap();
        let code_bytes = code.buffer.data();
        let code_size = code_bytes.len();

        // Allocate executable memory and copy code
        let executable_ptr = self.allocate_executable_memory(code_size)?;
        unsafe {
            std::ptr::copy_nonoverlapping(code_bytes.as_ptr(), executable_ptr, code_size);
        }

        Ok(Module::new(executable_ptr, code_size, program.len()))
    }

    /// Allocate executable memory using mmap
    fn allocate_executable_memory(&self, size: usize) -> Result<*mut u8> {
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
}
