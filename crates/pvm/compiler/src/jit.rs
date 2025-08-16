//! Clean block-based JIT compiler for PVM programs

use crate::{
    constants::{
        access, context_offsets, exec_result, BITS_PER_WORD, EXTRA_PAGES_MARGIN,
        JUMP_ALIGNMENT_FACTOR, LINEAR_MEMORY_SIZE, PAGE_SIZE, PVM_REGISTER_COUNT,
    },
    translator::Translator,
    Info, Memory, Module,
};
use anyhow::Result;
use cranelift::prelude::*;
use cranelift_codegen::ir::{Function, UserFuncName};
use std::collections::HashMap;

/// Basic block - single entry/exit instruction sequence
#[derive(Debug, Clone)]
pub struct Block {
    pub start: usize,
    pub end: usize,
    pub terminates: bool,
}

/// Compiled native code
#[derive(Debug, Clone)]
pub struct Code {
    pub ptr: *const u8,
    pub size: usize,
}

/// Block execution result
#[derive(Debug, Clone)]
pub enum ExecResult {
    Continue,
    Jump(u64),
    Halt,
    Trap,
}

/// Runtime context for block execution
#[derive(Debug, Clone)]
pub struct Context {
    pub registers: [u64; PVM_REGISTER_COUNT],
    pub pc: u64,
    pub memory: Memory,
    pub linear_mem: Vec<u8>,
}

impl Context {
    /// Create new context
    pub fn new(regs: [u64; PVM_REGISTER_COUNT], pc: u64, mem: Memory) -> Self {
        let mut linear_mem = vec![0u8; LINEAR_MEMORY_SIZE];

        // Copy memory pages to linear buffer
        for (&page_num, page) in &mem.pages {
            let start = (page_num as usize) * (PAGE_SIZE as usize);
            let end = start + page.data.len();
            if end <= linear_mem.len() {
                linear_mem[start..end].copy_from_slice(&page.data);
            }
        }

        Self {
            registers: regs,
            pc,
            memory: mem,
            linear_mem,
        }
    }

    /// Generate page allocation bitmap for runtime boundary checking
    pub fn generate_page_bitmap(&self) -> (Vec<u64>, Vec<u8>) {
        let max_page = self.memory.pages.keys().max().copied().unwrap_or(0);
        let bitmap_size = ((max_page + BITS_PER_WORD as u32) / BITS_PER_WORD as u32) as usize;
        let mut bitmap = vec![0u64; bitmap_size];

        // Ensure access array is large enough to handle boundary checking beyond max_page
        // We need to account for multi-byte stores that may access pages beyond max_page
        let access_size = (max_page + EXTRA_PAGES_MARGIN + 1) as usize;
        let mut access = vec![access::INACCESSIBLE; access_size]; // Default: inaccessible

        tracing::debug!(
            "Page bitmap generation: max_page={}, bitmap_size={}, access_size={}",
            max_page,
            bitmap_size,
            access_size
        );
        tracing::debug!(
            "Allocated pages: {:?}",
            self.memory.pages.keys().collect::<Vec<_>>()
        );

        for (&page_num, page) in &self.memory.pages {
            let word_idx = page_num / BITS_PER_WORD as u32;
            let bit_idx = page_num % BITS_PER_WORD as u32;
            tracing::debug!(
                "Page {}: word_idx={}, bit_idx={}, access={}",
                page_num,
                word_idx,
                bit_idx,
                page.access
            );
            if (word_idx as usize) < bitmap.len() {
                bitmap[word_idx as usize] |= 1u64 << bit_idx;
                if (page_num as usize) < access.len() {
                    access[page_num as usize] = page.access;
                }
            }
        }

        tracing::debug!("Final bitmap: {:?}", bitmap);
        tracing::debug!(
            "Access array length: {}, sample: {:?}",
            access.len(),
            &access[..access.len().min(10)]
        );

        (bitmap, access)
    }

    /// Sync linear memory back to pages
    ///
    /// NOTE: This method only detects page faults for writes that actually occurred.
    /// It cannot detect cases where a store instruction should have written more bytes
    /// but was truncated due to page boundaries. For proper page fault detection,
    /// boundary checking should be implemented in the store visitor functions.
    pub fn sync(&mut self) -> Result<()> {
        let page_size = PAGE_SIZE as usize;

        // Check for any writes to unallocated pages
        for page_addr in (0..self.linear_mem.len()).step_by(page_size) {
            let page_num = (page_addr / page_size) as u32;
            let page_end = (page_addr + page_size).min(self.linear_mem.len());

            if !self.memory.pages.contains_key(&page_num) {
                let page_data = &self.linear_mem[page_addr..page_end];
                if page_data.iter().any(|&b| b != 0) {
                    anyhow::bail!("Page fault: write to unallocated page {}", page_num);
                }
            }
        }

        // Check for read-only violations and copy back changes
        for (&page_num, page) in &mut self.memory.pages {
            let start = (page_num as usize) * page_size;
            let end = start + page.data.len();

            if end <= self.linear_mem.len() {
                let orig = &page.data[..];
                let new = &self.linear_mem[start..end];

                if orig != new {
                    // Check for read-only page violations
                    if page.access != access::MUTABLE {
                        anyhow::bail!("Page fault: write to read-only page {}", page_num);
                    }

                    // Copy changes back
                    page.data.copy_from_slice(new);
                }
            }
        }

        Ok(())
    }
}

/// Extended context for compiled blocks
#[repr(C)]
pub struct ExtendedContext {
    pub registers: [u64; PVM_REGISTER_COUNT],
    pub pc: u64,
    pub memory_ptr: *mut u8,
    pub page_bitmap: *const u64, // Bitmap of allocated pages
    pub page_access: *const u8,  // Access permissions per page
    pub result: ExecResult,
    pub pc_managed: bool, // Flag indicating instruction handled PC directly
}

/// JIT compiler
pub struct Jit {
    code_cache: HashMap<u64, Code>,
    blocks: HashMap<u64, Block>,
    jump_table: Vec<u64>,
    program: Vec<u8>,
    program_end_pc: u64, // PC at the end of the entire program
    ctx: cranelift_codegen::Context,
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
            code_cache: HashMap::new(),
            blocks: HashMap::new(),
            jump_table: Vec::new(),
            program: Vec::new(),
            program_end_pc: 0,
            ctx: cranelift_codegen::Context::new(),
            isa,
        })
    }

    /// Compile program - creates Module for compatibility
    pub fn compile(&mut self, program: &[u8]) -> Result<Module> {
        let has_trap = self.has_trap(program)?;
        Ok(
            Module::new(std::ptr::null(), 0, program.len(), has_trap)
                .with_program(program.to_vec()),
        )
    }

    /// Analyze program - discovers all basic blocks using read_block()
    /// Uses parser's natural block discovery for clean, efficient block creation
    pub fn analyze(&mut self, program: &[u8]) -> Result<()> {
        self.program = program.to_vec();
        let blob = parser::program::deblob(program)?;
        self.jump_table = blob.jump_table.clone();
        self.blocks.clear();

        let mut reader = blob.reader();
        let mut last_instruction_pc = 0;

        // Use read_block() to naturally discover block boundaries
        while !reader.eof() {
            let block_start = reader.position;
            let block_instructions = reader.read_block()?;

            if block_instructions.is_empty() {
                break;
            }

            // Track the end PC of the last instruction in the program
            if let Some(last_instr) = block_instructions.last() {
                last_instruction_pc = last_instr.range.end as u64;
                tracing::trace!(
                    "BLOCK {}-{}: last instruction at PC {} ({}), terminates={}",
                    block_start,
                    reader.position,
                    last_instr.range.start,
                    last_instr.value,
                    reader.eof()
                );
            }

            // Block terminates if it contains a terminating instruction OR if we reached EOF
            let terminates = !block_instructions.is_empty()
                && (reader.eof()
                    || matches!(
                        block_instructions.last().unwrap().value,
                        parser::Instruction::Trap
                            | parser::Instruction::Fallthrough
                            | parser::Instruction::Jump(_)
                            | parser::Instruction::JumpInd(_)
                            | parser::Instruction::LoadImmJump(_)
                            | parser::Instruction::LoadImmJumpInd(_)
                            | parser::Instruction::BranchEq(_)
                            | parser::Instruction::BranchNe(_)
                            | parser::Instruction::BranchGeU(_)
                            | parser::Instruction::BranchGeS(_)
                            | parser::Instruction::BranchLtU(_)
                            | parser::Instruction::BranchLtS(_)
                            | parser::Instruction::BranchEqImm(_)
                            | parser::Instruction::BranchNeImm(_)
                            | parser::Instruction::BranchGeUImm(_)
                            | parser::Instruction::BranchGeSImm(_)
                            | parser::Instruction::BranchLtUImm(_)
                            | parser::Instruction::BranchLtSImm(_)
                            | parser::Instruction::BranchLeUImm(_)
                            | parser::Instruction::BranchLeSImm(_)
                            | parser::Instruction::BranchGtUImm(_)
                            | parser::Instruction::BranchGtSImm(_)
                    ));

            self.create_block(block_start, reader.position, terminates);

            // Handle indirect jump table targets
            self.process_jump_targets(&block_instructions, &blob)?;
        }

        // Store the end PC of the last instruction in the entire program
        self.program_end_pc = last_instruction_pc;
        tracing::trace!("PROGRAM END PC set to: {}", self.program_end_pc);

        Ok(())
    }

    /// Create a block and insert it into the blocks map
    fn create_block(&mut self, start: usize, end: usize, terminates: bool) {
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
                            let _target_instructions = target_reader.read_block()?;
                            let target_end = target_reader.position;

                            self.create_block(target_start, target_end, true);
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
        loop {
            tracing::debug!("Executing block at PC: {}", ctx.pc);
            let code = self.get_code(ctx.pc)?;
            let block = self.blocks.get(&ctx.pc).cloned();
            let (result, _pc_managed) = self.run_block(&code, &mut ctx, block.clone())?;

            tracing::debug!("Block execution result: {:?}, new PC: {}", result, ctx.pc);
            tracing::trace!(
                "Block info: start={}, end={}, terminates={}",
                block.as_ref().map(|b| b.start).unwrap_or(0),
                block.as_ref().map(|b| b.end).unwrap_or(0),
                block.as_ref().map(|b| b.terminates).unwrap_or(false)
            );
            match result {
                ExecResult::Continue => {
                    // Continue to the next block
                    if let Some(current_block) = block {
                        // Find next sequential block using the current block's end
                        if let Some(next_pc) = self
                            .blocks
                            .keys()
                            .find(|&&p| p == current_block.end as u64)
                            .copied()
                        {
                            ctx.pc = next_pc;
                        } else {
                            // EOF case - no more blocks, use program end PC
                            tracing::debug!(
                                "EOF case: setting PC from {} to program_end_pc {}",
                                ctx.pc,
                                self.program_end_pc
                            );
                            ctx.pc = self.program_end_pc;
                            break;
                        }
                    } else {
                        break;
                    }
                }
                ExecResult::Jump(target) => {
                    tracing::trace!("Jump: PC {} -> target {}", ctx.pc, target);
                    if self.blocks.contains_key(&target) {
                        ctx.pc = target;
                    } else {
                        tracing::trace!("No block at target PC {}, treating as halt", target);
                        ctx.pc = target;
                        break;
                    }
                }
                ExecResult::Halt => {
                    // PC should be at the halt instruction (e.g., jump_ind to zero)
                    // PC is already set correctly by the instruction visitor
                    break;
                }
                ExecResult::Trap => {
                    // PC should already be set correctly by the instruction visitor
                    // Don't override it
                    break;
                }
            }
        }

        Ok(Info {
            registers: ctx.registers,
            pc: ctx.pc,
            memory: ctx.memory,
        })
    }

    /// Get or compile code for PC
    fn get_code(&mut self, pc: u64) -> Result<Code> {
        if let Some(code) = self.code_cache.get(&pc).cloned() {
            return Ok(code);
        }

        let code = self.compile_block(pc)?;
        self.code_cache.insert(pc, code.clone());
        Ok(code)
    }

    /// Compile single basic block
    fn compile_block(&mut self, pc: u64) -> Result<Code> {
        let block = self
            .blocks
            .get(&pc)
            .ok_or_else(|| anyhow::anyhow!("No block at PC {}", pc))?
            .clone();

        let mut sig = Signature::new(self.isa.default_call_conv());
        sig.params.push(AbiParam::new(types::I64));

        let mut func = Function::with_name_signature(UserFuncName::user(0, pc as u32), sig);
        let mut builder_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut func, &mut builder_ctx);

        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        builder.seal_block(entry);

        let ctx_ptr = builder.block_params(entry)[0];

        match self.translate(&mut builder, ctx_ptr, &block) {
            Ok(_) => {
                builder.ins().return_(&[]);
                builder.finalize();

                self.ctx.clear();
                self.ctx.func = func;
                let mut ctrl = cranelift_codegen::control::ControlPlane::default();
                self.ctx
                    .compile(&*self.isa, &mut ctrl)
                    .map_err(|e| anyhow::anyhow!("Cranelift failed: {:?}", e))?;

                let code = self.ctx.compiled_code().unwrap();
                let bytes = code.buffer.data();
                let size = bytes.len();

                let ptr = self.alloc_exec(size)?;
                unsafe {
                    std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, size);
                }

                Ok(Code { ptr, size })
            }
            Err(e) => {
                tracing::warn!("Block compilation failed for PC {}: {}", pc, e);
                Ok(Code {
                    ptr: std::ptr::null(),
                    size: 0,
                })
            }
        }
    }

    /// Translate block using Translator
    fn translate(
        &self,
        builder: &mut FunctionBuilder,
        ctx_ptr: Value,
        block: &Block,
    ) -> Result<()> {
        let mut translator = Translator::new(builder);

        // Initialize translator with context pointer
        translator.init_with_context(ctx_ptr)?;

        // Load initial register values from context into Cranelift variables
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

        translator.translate_block(&self.program, block.start, block.end)?;

        // For non-terminating blocks, advance PC to end of block
        // Terminating instructions handle their own PC advancement
        if !block.terminates {
            let final_pc = translator.get_final_pc();
            let pc_offset = translator
                .builder
                .ins()
                .iconst(types::I64, context_offsets::PC_OFFSET as i64);
            let pc_addr = translator.builder.ins().iadd(ctx_ptr, pc_offset);
            let new_pc = translator.builder.ins().iconst(types::I64, final_pc as i64);
            translator
                .builder
                .ins()
                .store(MemFlags::new(), new_pc, pc_addr, 0);
        }

        // Save registers back to context
        for i in 0..PVM_REGISTER_COUNT {
            let reg_var = translator.registers[&(i as u8)];
            let reg_val = translator.builder.use_var(reg_var);
            let offset = translator.builder.ins().iconst(types::I64, (i * 8) as i64);
            let addr = translator.builder.ins().iadd(ctx_ptr, offset);
            translator
                .builder
                .ins()
                .store(MemFlags::new(), reg_val, addr, 0);
        }

        Ok(())
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
    fn run_block(
        &self,
        code: &Code,
        ctx: &mut Context,
        _block: Option<Block>,
    ) -> Result<(ExecResult, bool)> {
        if code.ptr.is_null() {
            return Ok((ExecResult::Continue, false));
        }

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
            let func = std::mem::transmute::<*const u8, fn(*mut ExtendedContext)>(code.ptr);
            func(&mut ext_ctx);
        }

        ctx.registers = ext_ctx.registers;
        ctx.pc = ext_ctx.pc;

        // Always decode the result - terminating instructions set their own results
        // Non-terminating instructions leave the default result (Continue)
        let result = self.decode_result(&ext_ctx)?;

        match ctx.sync() {
            Ok(_) => {
                tracing::trace!("No page fault detected, PC remains {}", ctx.pc);
                Ok((result, ext_ctx.pc_managed))
            }
            Err(e) => {
                tracing::trace!("Page fault detected: {}, setting PC to 0", e);
                ctx.pc = 0;
                Ok((ExecResult::Trap, false))
            }
        }
    }

    /// Decode execution result from context
    fn decode_result(&self, ext_ctx: &ExtendedContext) -> Result<ExecResult> {
        let offset = context_offsets::RESULT_OFFSET;

        unsafe {
            let ctx_ptr = ext_ctx as *const ExtendedContext as *const u8;
            let result_ptr = ctx_ptr.add(offset);
            let discriminant = *(result_ptr as *const u64);

            tracing::trace!("decode_result: discriminant={}", discriminant);
            match discriminant {
                exec_result::CONTINUE => {
                    tracing::trace!("Branch result: Continue (don't take branch)");
                    Ok(ExecResult::Continue)
                }
                exec_result::JUMP => {
                    let target = *(result_ptr.add(8) as *const u64);
                    tracing::trace!("Direct jump result: target PC {}", target);
                    Ok(ExecResult::Jump(target))
                }
                exec_result::HALT => Ok(ExecResult::Halt),
                exec_result::TRAP => Ok(ExecResult::Trap),
                exec_result::JUMP_INDIRECT => {
                    // JumpIndirect - resolve address through jump table
                    let address = *(result_ptr.add(8) as *const u64) as u32;
                    tracing::trace!(
                        "Indirect jump result: address {} (needs jump table resolution)",
                        address
                    );

                    // Implement djump logic from interpreter
                    if address == u32::MAX - u16::MAX as u32 {
                        return Ok(ExecResult::Halt);
                    }

                    // Jump to zero means halt
                    if address == 0 {
                        tracing::trace!("JumpInd to zero: returning Halt");
                        return Ok(ExecResult::Halt);
                    }

                    if address > self.jump_table.len() as u32 * JUMP_ALIGNMENT_FACTOR
                        || address % 2 != 0
                    {
                        tracing::trace!(
                            "Invalid dynamic jump: address={}, table_len={}",
                            address,
                            self.jump_table.len()
                        );
                        return Ok(ExecResult::Trap);
                    }

                    let index = address as usize / 2 - 1;
                    if let Some(&target_pc) = self.jump_table.get(index) {
                        tracing::trace!(
                            "Resolved indirect jump: address {} -> index {} -> PC {}",
                            address,
                            index,
                            target_pc
                        );
                        Ok(ExecResult::Jump(target_pc))
                    } else {
                        tracing::trace!(
                            "Jump table index {} out of bounds (table_len={})",
                            index,
                            self.jump_table.len()
                        );
                        Ok(ExecResult::Trap)
                    }
                }
                _ => Ok(ExecResult::Continue),
            }
        }
    }

    /// Check if program has trap instructions
    fn has_trap(&self, program: &[u8]) -> Result<bool> {
        let blob = parser::program::deblob(program)?;
        let mut reader = blob.reader();

        while !reader.eof() {
            let block_instructions = reader.read_block()?;

            // Check all instructions in the block for trap instructions
            for instr in block_instructions {
                if matches!(instr.value, parser::Instruction::Trap) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}
