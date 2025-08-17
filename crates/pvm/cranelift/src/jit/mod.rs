//! Clean block-based JIT compiler for PVM programs

use crate::{
    constants::PVM_REGISTER_COUNT,
    translator::{Block, Code, Translator},
    utils, Info,
};
use anyhow::Result;
use cranelift::prelude::*;
use cranelift_codegen::ir::{Function, UserFuncName};
use std::collections::HashMap;
pub use {
    context::{Context, ExtendedContext},
    result::ExecResult,
};

mod context;
mod result;

/// JIT compiler
pub struct Jit {
    /// Map of blocks by start PC
    blocks: HashMap<u64, Block>,
    /// Jump table for indirect jumps
    jump_table: Vec<u64>,
    /// Program bytes
    program: Vec<u8>,
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
            blocks: HashMap::new(),
            jump_table: Vec::new(),
            program: Vec::new(),
            ctx: cranelift_codegen::Context::new(),
            isa,
        })
    }

    /// Analyze program - discovers all basic blocks using read_block()
    /// Uses parser's natural block discovery for clean, efficient block creation
    pub fn analyze(&mut self, program: &[u8]) -> Result<bool> {
        self.program = program.to_vec();
        let blob = parser::program::deblob(program)?;
        self.jump_table = blob.jump_table.clone();
        self.blocks.clear();

        let mut reader = blob.reader();
        let mut has_trap = false;

        // Use read_block() to naturally discover block boundaries
        while !reader.eof() {
            let block_start = reader.position;
            let block_instructions = reader.read_block()?;

            if block_instructions.is_empty() {
                break;
            }

            // Block terminates if it contains a terminating instruction OR if we reached EOF
            let terminates = !block_instructions.is_empty()
                && (reader.eof()
                    || utils::is_terminating_instruction(
                        &block_instructions.last().unwrap().value,
                    ));

            // Handle indirect jump table targets first
            self.process_jump_targets(&block_instructions, &blob)?;

            // Check all instructions in the block for trap instructions
            for instr in &block_instructions {
                if matches!(instr.value, parser::Instruction::Trap) {
                    has_trap = true;
                }
            }

            // Only create block if it doesn't already exist (might have been created by process_jump_targets)
            if !self.blocks.contains_key(&(block_start as u64)) {
                self.create_block(block_start, reader.position, terminates, block_instructions);
            }
        }

        Ok(has_trap)
    }

    /// Create a block and insert it into the blocks map
    fn create_block(
        &mut self,
        start: usize,
        end: usize,
        terminates: bool,
        instructions: Vec<parser::reader::Offset<parser::Instruction>>,
    ) {
        tracing::trace!(
            "Block: start={}, end={}, terminates={}",
            start,
            end,
            terminates
        );

        self.blocks.insert(
            start as u64,
            Block {
                start,
                end,
                terminates,
                instructions,
            },
        );
    }

    /// Process jump targets from indirect jump instructions
    fn process_jump_targets(
        &mut self,
        block_instructions: &[parser::reader::Offset<parser::Instruction>],
        blob: &parser::program::ProgramBlob,
    ) -> Result<()> {
        if let Some(last_instruction) = block_instructions.last() {
            if matches!(
                last_instruction.value,
                parser::Instruction::JumpInd(_) | parser::Instruction::LoadImmJumpInd(_)
            ) {
                // Clone jump_table to avoid borrow checker issues
                let jump_table = self.jump_table.clone();
                for &target in &jump_table {
                    if (target as usize) < blob.instructions.len()
                        && !self.blocks.contains_key(&target)
                    {
                        let mut target_reader = blob.reader();
                        target_reader.set_position(target as usize);

                        if !target_reader.eof() {
                            let target_start = target_reader.position;
                            let target_instructions = target_reader.read_block()?;
                            let target_end = target_reader.position;

                            // Check if the block actually terminates (has a terminating instruction)
                            let terminates = !target_instructions.is_empty()
                                && utils::is_terminating_instruction(
                                    &target_instructions.last().unwrap().value,
                                );

                            self.create_block(
                                target_start,
                                target_end,
                                terminates,
                                target_instructions,
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Execute program using JIT compilation
    pub fn execute(&mut self, mut ctx: Context) -> Result<Info> {
        tracing::debug!("Starting execution with initial PC: {}", ctx.pc);
        let unified_code = self.compile_unified_program()?;
        self.run(&unified_code, &mut ctx)?;

        Ok(Info {
            registers: ctx.registers,
            pc: ctx.pc,
            memory: ctx.memory.clone(),
        })
    }

    /// Compile entire program as unified Cranelift function (cranelift-wasm style)
    fn compile_unified_program(&mut self) -> Result<Code> {
        tracing::debug!("Compiling entire program as unified Cranelift function");
        let mut flag_builder = settings::builder();
        flag_builder
            .set("use_colocated_libcalls", "false")
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        flag_builder
            .set("is_pic", "false")
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let isa_builder = cranelift_native::builder().map_err(|e| anyhow::anyhow!("{}", e))?;
        let isa = isa_builder.finish(settings::Flags::new(flag_builder))?;

        let mut sig = Signature::new(isa.default_call_conv());
        sig.params.push(AbiParam::new(types::I64)); // context pointer
        sig.params.push(AbiParam::new(types::I64)); // starting PC
        let mut func = Function::with_name_signature(UserFuncName::user(0, 0), sig);
        let mut builder_ctx = FunctionBuilderContext::new();
        let mut translator = Translator::new(&mut func, &mut builder_ctx, self.jump_table.clone())?;

        // Step 1: Create all Cranelift blocks upfront (enables forward jumps)
        let mut block_map = std::collections::HashMap::new();
        for &pc in self.blocks.keys() {
            let cranelift_block = translator.builder.create_block();
            block_map.insert(pc, cranelift_block);
            tracing::trace!("Created Cranelift block for PVM PC {}", pc);
        }

        translator.init_blocks(block_map);

        // Create entry block
        let entry = translator.builder.create_block();
        translator
            .builder
            .append_block_params_for_function_params(entry);
        translator.builder.switch_to_block(entry);

        let ctx_ptr = translator.builder.block_params(entry)[0];
        let start_pc = translator.builder.block_params(entry)[1];

        // Use scoped translator to avoid ownership issues
        {
            translator.init_with_context(ctx_ptr)?;

            // Load all registers from context ONCE at function entry
            for i in 0..PVM_REGISTER_COUNT {
                let reg_var = translator.registers[&(i as u8)];
                let offset = translator.builder.ins().iconst(types::I64, (i * 8) as i64);
                let addr = translator.builder.ins().iadd(ctx_ptr, offset);
                let reg_val = translator
                    .builder
                    .ins()
                    .load(types::I64, MemFlags::new(), addr, 0);
                translator.builder.def_var(reg_var, reg_val);
            }

            // Dispatcher: jump to the requested starting PC
            // Create a switch statement to jump to the correct block based on start_pc
            let mut switch = cranelift::frontend::Switch::new();
            for (&pc, &cranelift_block) in &translator.blocks {
                switch.set_entry(pc as u128, cranelift_block);
            }

            // Default case: if PC is not found, return with trap
            let default_block = translator.builder.create_block();
            translator.builder.switch_to_block(default_block);
            translator.return_trap()?;
            translator.builder.seal_block(default_block);

            // Generate the switch on start_pc
            translator.builder.switch_to_block(entry);
            switch.emit(&mut translator.builder, start_pc, default_block);
            translator.builder.seal_block(entry);

            // Step 2: Translate all PVM blocks to Cranelift basic blocks using shared translator
            for (&pc, pvm_block) in &self.blocks {
                let cranelift_block = translator.blocks[&pc];
                translator.builder.switch_to_block(cranelift_block);

                tracing::trace!("Translating PVM block at PC {} to Cranelift block", pc);

                // Translate instructions in this block using shared translator
                match translator.translate_block(pvm_block) {
                    Ok(_) => {
                        tracing::trace!("Successfully translated block at PC {}", pc);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to translate block at PC {}: {}", pc, e);
                        // Generate trap for failed blocks
                        translator.return_trap()?;
                    }
                }
            }

            // Step 3: Seal all blocks after translation
            for &cranelift_block in translator.blocks.values() {
                translator.builder.seal_block(cranelift_block);
            }
        } // translator goes out of scope here

        // Finalize the function
        translator.builder.finalize();

        self.ctx.clear();
        self.ctx.func = func;
        let mut ctrl = cranelift_codegen::control::ControlPlane::default();
        self.ctx
            .compile(&*self.isa, &mut ctrl)
            .map_err(|e| anyhow::anyhow!("Unified compilation failed: {:?}", e))?;

        let code = self.ctx.compiled_code().unwrap();
        let bytes = code.buffer.data();
        let size = bytes.len();

        let ptr = self.alloc_exec(size)?;
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, size);
        }

        tracing::debug!("Unified compilation completed, generated {} bytes", size);
        Ok(Code { ptr, size })
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

    /// Execute compiled block
    /// Run the unified compiled function - executes entire program
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
            // Call the unified function with starting PC
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
                tracing::trace!("Unified execution completed, final PC: {}", ctx.pc);
                Ok(())
            }
            Err(e) => {
                tracing::trace!("Page fault detected during unified execution: {}", e);
                ctx.pc = 0;
                Ok(())
            }
        }
    }
}
