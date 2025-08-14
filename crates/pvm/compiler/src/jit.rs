//! Clean block-based JIT compiler for PVM programs

use crate::{translator::Translator, Info, Memory, Module};
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
    pub registers: [u64; 13],
    pub pc: u64,
    pub memory: Memory,
    pub linear_mem: Vec<u8>,
}

impl Context {
    /// Create new context
    pub fn new(regs: [u64; 13], pc: u64, mem: Memory) -> Self {
        let mut linear_mem = vec![0u8; 0x100000]; // 1MB

        // Copy memory pages to linear buffer
        for (&page_num, page) in &mem.pages {
            let start = (page_num as usize) * (crate::module::memory::PAGE_SIZE as usize);
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
        let bitmap_size = ((max_page + 64) / 64) as usize; // Round up to u64 chunks
        let mut bitmap = vec![0u64; bitmap_size];
        let mut access = vec![2u8; (max_page + 1) as usize]; // Default: inaccessible (non-zero)

        for (&page_num, page) in &self.memory.pages {
            let word_idx = page_num / 64;
            let bit_idx = page_num % 64;
            if (word_idx as usize) < bitmap.len() {
                bitmap[word_idx as usize] |= 1u64 << bit_idx;
                if (page_num as usize) < access.len() {
                    access[page_num as usize] = page.access;
                }
            }
        }

        (bitmap, access)
    }

    /// Sync linear memory back to pages
    ///
    /// NOTE: This method only detects page faults for writes that actually occurred.
    /// It cannot detect cases where a store instruction should have written more bytes
    /// but was truncated due to page boundaries. For proper page fault detection,
    /// boundary checking should be implemented in the store visitor functions.
    pub fn sync(&mut self) -> Result<()> {
        let page_size = crate::module::memory::PAGE_SIZE as usize;

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
                    if page.access != 0 {
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
    pub registers: [u64; 13],
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
    ctx: cranelift_codegen::Context,
    isa: cranelift_codegen::isa::OwnedTargetIsa,
}

impl Jit {
    /// Create new JIT compiler
    /// EXISTS: Entry point for JIT compilation
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
            ctx: cranelift_codegen::Context::new(),
            isa,
        })
    }

    /// Compile program - creates Module for compatibility
    /// EXISTS: Required by Module interface
    pub fn compile(&mut self, program: &[u8]) -> Result<Module> {
        let has_trap = self.has_trap(program)?;
        Ok(
            Module::new(std::ptr::null(), 0, program.len(), has_trap)
                .with_program(program.to_vec()),
        )
    }

    /// Analyze program - discovers all basic blocks using Graypaper formula
    /// EXISTS: Core function - implements π ≡ ({0} ∪ {n + 1 + skip(n) | ...}) formula
    pub fn analyze(&mut self, program: &[u8]) -> Result<()> {
        self.program = program.to_vec();

        let blob = parser::program::deblob(program)?;
        self.jump_table = blob.jump_table.clone();

        let mut starts = std::collections::BTreeSet::new();
        starts.insert(0); // Graypaper: {0}

        let mut reader = blob.reader();
        while !reader.eof() {
            let instruction_offset = match reader.read() {
                Ok(instr) => instr,
                Err(_) => break,
            };

            if self.instruction_terminates(&instruction_offset.value) {
                // Add next instruction start (after this terminating instruction)
                let next_start = reader.position;
                if next_start < blob.instructions.len() {
                    starts.insert(next_start);
                }

                // Add jump targets for indirect jumps using parsed instructions
                match instruction_offset.value {
                    parser::Instruction::JumpInd(_) | parser::Instruction::LoadImmJumpInd(_) => {
                        for &target in &self.jump_table {
                            if (target as usize) < blob.instructions.len() {
                                starts.insert(target as usize);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        self.create_blocks(starts)
    }

    /// Execute program using JIT compilation
    /// EXISTS: Main execution loop - compiles blocks on demand and handles control flow
    pub fn execute(&mut self, mut ctx: Context) -> Result<Info> {
        loop {
            let code = self.get_code(ctx.pc)?;
            let block = self.blocks.get(&ctx.pc).cloned();
            let (result, _pc_managed) = self.run_block(&code, &mut ctx, block.clone())?;

            match result {
                ExecResult::Continue => {
                    // For terminating blocks (like branch instructions), advance to next instruction and STOP
                    if let Some(block) = block {
                        if block.terminates {
                            // Branch instruction chose Continue - advance to next instruction and halt
                            let next_pc = self.next_instruction_pc(ctx.pc)?;
                            tracing::trace!("Continue: advancing from PC {} to next_instruction_pc {} and stopping", ctx.pc, next_pc);
                            ctx.pc = next_pc;
                            break; // Stop execution after branch Continue
                        } else {
                            // Non-terminating block - find next block
                            if let Some(next_pc) = self.next_block(ctx.pc) {
                                ctx.pc = next_pc;
                            } else {
                                break;
                            }
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
                ExecResult::Halt => break,
                ExecResult::Trap => {
                    // For terminating blocks, advance PC to the terminating instruction before trapping
                    if let Some(block) = block {
                        if block.terminates {
                            let terminating_pc = self.terminating_instruction_pc(ctx.pc)?;
                            tracing::trace!(
                                "Trap: advancing to terminating instruction PC {}",
                                terminating_pc
                            );
                            ctx.pc = terminating_pc;
                        }
                    }
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

    /// Check if instruction is terminating (c_n ∈ T) using parsed instruction
    /// EXISTS: Core Graypaper requirement - determines set T (terminating instructions)  
    fn instruction_terminates(&self, instruction: &parser::Instruction) -> bool {
        matches!(
            instruction,
            parser::Instruction::Trap
                | parser::Instruction::Fallthrough
                | parser::Instruction::Jump(_)
                | parser::Instruction::JumpInd(_)
                | parser::Instruction::LoadImmJump(_)
                | parser::Instruction::LoadImmJumpInd(_)
                | parser::Instruction::BranchEq(_)
                | parser::Instruction::BranchNe(_)
                | parser::Instruction::BranchLtU(_)
                | parser::Instruction::BranchLtS(_)
                | parser::Instruction::BranchGeU(_)
                | parser::Instruction::BranchGeS(_)
                | parser::Instruction::BranchEqImm(_)
                | parser::Instruction::BranchNeImm(_)
                | parser::Instruction::BranchLtUImm(_)
                | parser::Instruction::BranchLtSImm(_)
                | parser::Instruction::BranchGeUImm(_)
                | parser::Instruction::BranchGeSImm(_)
                | parser::Instruction::BranchLeUImm(_)
                | parser::Instruction::BranchLeSImm(_)
                | parser::Instruction::BranchGtUImm(_)
                | parser::Instruction::BranchGtSImm(_)
        )
    }

    /// Create blocks from boundary set
    /// EXISTS: Converts Graypaper formula result to Block structures
    fn create_blocks(&mut self, starts: std::collections::BTreeSet<usize>) -> Result<()> {
        let vec: Vec<_> = starts.into_iter().collect();

        for i in 0..vec.len() {
            let start = vec[i];
            let end = if i + 1 < vec.len() {
                vec[i + 1]
            } else {
                self.program.len()
            };

            if start >= end {
                continue;
            }

            // Block terminates if there's a next block (Graypaper formula guarantees this)
            let terminates = i + 1 < vec.len() || self.last_terminates(start, end)?;

            tracing::trace!(
                "Block {}: start={}, end={}, terminates={}",
                i, start, end, terminates
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

        Ok(())
    }

    /// Check if last block terminates
    /// EXISTS: Only needed for final block in program (edge case)
    fn last_terminates(&self, start: usize, end: usize) -> Result<bool> {
        if start >= end {
            return Ok(false);
        }

        let blob = parser::program::deblob(&self.program)?;
        let mut reader = blob.reader();
        reader.set_position(start);

        let mut last_instruction = None;

        while !reader.eof() && reader.position < end {
            let instruction_offset = match reader.read() {
                Ok(instr) => instr,
                Err(_) => break,
            };

            if instruction_offset.range.start >= end {
                break;
            }

            last_instruction = Some(instruction_offset.value);
        }

        Ok(last_instruction.map_or(false, |instr| self.instruction_terminates(&instr)))
    }

    /// Get or compile code for PC
    /// EXISTS: Performance optimization - cache compiled blocks
    fn get_code(&mut self, pc: u64) -> Result<Code> {
        if let Some(code) = self.code_cache.get(&pc).cloned() {
            return Ok(code);
        }

        let code = self.compile_block(pc)?;
        self.code_cache.insert(pc, code.clone());
        Ok(code)
    }

    /// Compile single basic block
    /// EXISTS: Core JIT functionality - translates PVM to native code
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
    /// EXISTS: Delegates to shared translator between compiler/interpreter
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
        for i in 0..13 {
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

        // Only do block-level PC advancement for non-terminating blocks
        // Terminating instructions (branches/jumps) handle their own PC advancement
        if !block.terminates {
            let final_pc = translator.get_final_pc();
            let pc_offset = translator.builder.ins().iconst(types::I64, (13 * 8) as i64); // PC is after 13 registers
            let pc_addr = translator.builder.ins().iadd(ctx_ptr, pc_offset);
            let new_pc = translator.builder.ins().iconst(types::I64, final_pc as i64);
            translator
                .builder
                .ins()
                .store(MemFlags::new(), new_pc, pc_addr, 0);
        }

        // Save registers back to context
        for i in 0..13 {
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
    /// EXISTS: Required for JIT - need executable pages for generated code
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

    /// Find next sequential block
    /// EXISTS: Handles Continue result - finds next block in sequence
    fn next_block(&self, pc: u64) -> Option<u64> {
        let block = self.blocks.get(&pc)?;
        self.blocks
            .keys()
            .find(|&&p| p == block.end as u64)
            .copied()
    }
    /// Calculate PC of the terminating instruction in a block
    /// This finds the PC where the terminating instruction starts
    fn terminating_instruction_pc(&self, pc: u64) -> Result<u64> {
        if let Some(block) = self.blocks.get(&pc) {
            let blob = parser::program::deblob(&self.program)?;
            let mut reader = blob.reader();
            reader.set_position(block.start);

            let mut last_instruction_start = block.start;

            // Find the last (terminating) instruction in this block
            while !reader.eof() && reader.position < block.end {
                let instruction_offset = reader.read()?;
                if instruction_offset.range.start >= block.end {
                    break;
                }
                // Update to the start of this instruction (the terminating one will be the last)
                last_instruction_start = instruction_offset.range.start;
            }

            Ok(last_instruction_start as u64)
        } else {
            anyhow::bail!("No block found at PC {}", pc)
        }
    }

    /// Calculate next instruction PC for Continue results
    /// This finds the PC of the instruction immediately after the current one
    fn next_instruction_pc(&self, pc: u64) -> Result<u64> {
        // Find the last (terminating) instruction in the block and advance past it
        if let Some(block) = self.blocks.get(&pc) {
            let blob = parser::program::deblob(&self.program)?;
            let mut reader = blob.reader();
            reader.set_position(block.start);

            let mut last_instruction_end = block.start;

            // Find the last instruction in this block
            while !reader.eof() && reader.position < block.end {
                let instruction_offset = reader.read()?;
                if instruction_offset.range.start >= block.end {
                    break;
                }
                // Update to the end of this instruction
                last_instruction_end = reader.position;
            }

            Ok(last_instruction_end as u64)
        } else {
            anyhow::bail!("No block found at PC {}", pc)
        }
    }

    /// Execute compiled block
    /// EXISTS: Core execution - calls native code and handles results
    fn run_block(
        &self,
        code: &Code,
        ctx: &mut Context,
        block: Option<Block>,
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

        let result = if let Some(ref block) = block {
            if block.terminates
                && (self.block_contains_branch(block)? || self.block_contains_jump(block)?)
            {
                self.decode_result(&ext_ctx)?
            } else {
                ExecResult::Continue
            }
        } else {
            ExecResult::Continue
        };

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
    /// EXISTS: Required to extract result from compiled block
    fn decode_result(&self, ext_ctx: &ExtendedContext) -> Result<ExecResult> {
        let offset = std::mem::size_of::<[u64; 13]>()
            + std::mem::size_of::<u64>()
            + std::mem::size_of::<*mut u8>();

        unsafe {
            let ctx_ptr = ext_ctx as *const ExtendedContext as *const u8;
            let result_ptr = ctx_ptr.add(offset);
            let discriminant = *(result_ptr as *const u64);

            tracing::trace!("decode_result: discriminant={}", discriminant);
            match discriminant {
                0 => {
                    tracing::trace!("Branch result: Continue (don't take branch)");
                    Ok(ExecResult::Continue)
                }
                1 => {
                    let target = *(result_ptr.add(8) as *const u64);
                    tracing::trace!("Direct jump result: target PC {}", target);
                    Ok(ExecResult::Jump(target))
                }
                2 => Ok(ExecResult::Halt),
                3 => Ok(ExecResult::Trap),
                4 => {
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

                    // Jump alignment factor from interpreter implementation
                    const JUMP_ALIGNMENT_FACTOR: u32 = 2;
                    if address == 0
                        || address > self.jump_table.len() as u32 * JUMP_ALIGNMENT_FACTOR
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
    /// EXISTS: Required by Module interface
    fn has_trap(&self, program: &[u8]) -> Result<bool> {
        let blob = parser::program::deblob(program)?;
        let mut reader = blob.reader();

        while !reader.eof() {
            let instr = reader.read()?;
            if matches!(instr.value, parser::Instruction::Trap) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Get discovered blocks (testing)
    /// EXISTS: Required by test interface
    pub fn get_basic_blocks(&self) -> &HashMap<u64, Block> {
        &self.blocks
    }

    /// Check if block contains branch instructions
    fn block_contains_branch(&self, block: &Block) -> Result<bool> {
        let blob = parser::program::deblob(&self.program)?;
        let mut reader = blob.reader();
        reader.set_position(block.start);

        while !reader.eof() && reader.position < block.end {
            let instruction_offset = reader.read()?;
            if instruction_offset.range.start >= block.end {
                break;
            }

            // Check if this is a branch instruction using the parsed instruction
            match instruction_offset.value {
                parser::Instruction::BranchEq(_)
                | parser::Instruction::BranchNe(_)
                | parser::Instruction::BranchLtU(_)
                | parser::Instruction::BranchLtS(_)
                | parser::Instruction::BranchGeU(_)
                | parser::Instruction::BranchGeS(_)
                | parser::Instruction::BranchEqImm(_)
                | parser::Instruction::BranchNeImm(_)
                | parser::Instruction::BranchLtUImm(_)
                | parser::Instruction::BranchLtSImm(_)
                | parser::Instruction::BranchGeUImm(_)
                | parser::Instruction::BranchGeSImm(_)
                | parser::Instruction::BranchLeUImm(_)
                | parser::Instruction::BranchLeSImm(_)
                | parser::Instruction::BranchGtUImm(_)
                | parser::Instruction::BranchGtSImm(_) => {
                    return Ok(true);
                }
                _ => {}
            }
        }

        Ok(false)
    }

    /// Check if block contains jump instructions  
    fn block_contains_jump(&self, block: &Block) -> Result<bool> {
        let blob = parser::program::deblob(&self.program)?;
        let mut reader = blob.reader();
        reader.set_position(block.start);

        while !reader.eof() && reader.position < block.end {
            let instruction_offset = reader.read()?;
            if instruction_offset.range.start >= block.end {
                break;
            }

            // Check if this is a jump instruction using the parsed instruction
            match instruction_offset.value {
                parser::Instruction::Jump(_)
                | parser::Instruction::JumpInd(_)
                | parser::Instruction::LoadImmJump(_)
                | parser::Instruction::LoadImmJumpInd(_) => {
                    return Ok(true);
                }
                _ => {}
            }
        }

        Ok(false)
    }
}
