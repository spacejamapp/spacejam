//! Block-based JIT compiler for PVM programs
//!
//! This module implements a block-based JIT compilation strategy where PVM programs
//! are broken down into basic blocks and compiled incrementally on-demand.

use crate::{Memory, Info, Module, translator::Translator};
use anyhow::Result;
use cranelift::prelude::*;
use cranelift_codegen::ir::{Function, UserFuncName};
use parser::Instruction;
use std::collections::HashMap;

/// A basic block represents a sequence of instructions with a single entry and exit point
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Starting PC offset of the block
    pub start_pc: usize,
    /// Ending PC offset of the block (exclusive)  
    pub end_pc: usize,
    /// Whether this block ends with a terminating instruction (jump, branch, trap)
    pub is_terminating: bool,
    /// Compiled native code for this block (None if not yet compiled)
    pub compiled_code: Option<CompiledBlock>,
}

/// Compiled native code for a basic block
#[derive(Debug, Clone)]
pub struct CompiledBlock {
    /// Entry point address of the compiled block
    pub entry_point: *const u8,
    /// Size of the compiled block in bytes
    pub size: usize,
}

/// Result of executing a basic block
#[derive(Debug, Clone)]
pub enum BlockExecutionResult {
    /// Continue execution to the next sequential block
    Continue,
    /// Jump to a specific PC address
    Jump(u64),
    /// Halt execution (normal termination)
    Halt,
    /// Trap execution (error/panic termination) 
    Trap,
}

/// Execution context shared between blocks
#[derive(Debug, Clone)]
pub struct BlockContext {
    /// PVM registers (13 registers)
    pub registers: [u64; 13],
    /// Program counter
    pub pc: u64,
    /// Memory state
    pub memory: Memory,
    /// Linear memory buffer for JIT access (1MB)
    /// This is synced with Memory pages before/after block execution
    pub linear_memory: Vec<u8>,
}

impl BlockContext {
    /// Create a new block context with initial state
    pub fn new(registers: [u64; 13], pc: u64, memory: Memory) -> Self {
        // Initialize linear memory buffer
        let mut linear_memory = vec![0u8; 0x100000]; // 1MB
        
        // Copy memory pages to linear buffer
        for (&page_num, page) in &memory.pages {
            let start_addr = (page_num as usize) * (crate::module::memory::PAGE_SIZE as usize);
            let end_addr = start_addr + page.data.len();
            if end_addr <= linear_memory.len() {
                linear_memory[start_addr..end_addr].copy_from_slice(&page.data);
            }
        }
        
        Self {
            registers,
            pc,
            memory,
            linear_memory,
        }
    }
    
    /// Sync linear memory back to Memory pages
    pub fn sync_memory_from_linear(&mut self) {
        for (&page_num, page) in &mut self.memory.pages {
            let start_addr = (page_num as usize) * (crate::module::memory::PAGE_SIZE as usize);
            let end_addr = start_addr + page.data.len();
            if end_addr <= self.linear_memory.len() {
                page.data.copy_from_slice(&self.linear_memory[start_addr..end_addr]);
            }
        }
    }
    
    /// Sync linear memory back to Memory pages with validation
    /// Returns error if memory access violations are detected
    pub fn sync_memory_from_linear_with_validation(&mut self) -> Result<()> {
        // First, detect if any changes were made to unallocated memory regions
        // For efficient validation, we'll check if non-zero data exists in unallocated pages
        
        let page_size = crate::module::memory::PAGE_SIZE as usize;
        
        // Check each 4KB page in linear memory
        for page_addr in (0..self.linear_memory.len()).step_by(page_size) {
            let page_num = (page_addr / page_size) as u32;
            let page_end = (page_addr + page_size).min(self.linear_memory.len());
            
            // Check if this page is allocated
            if !self.memory.pages.contains_key(&page_num) {
                // Page is not allocated - check if any writes occurred to it
                let page_data = &self.linear_memory[page_addr..page_end];
                if page_data.iter().any(|&b| b != 0) {
                    anyhow::bail!("Page fault: write to unallocated page {}", page_num);
                }
            } else {
                // Page is allocated - check access permissions
                let page = &self.memory.pages[&page_num];
                if page.access != 0 {
                    // Page is not writable - check if any changes were made
                    let original_data = &page.data[..];
                    let new_data = &self.linear_memory[page_addr..page_end];
                    if original_data != new_data {
                        anyhow::bail!("Page fault: write to read-only page {}", page_num);
                    }
                }
            }
        }
        
        // If validation passed, perform the actual sync
        self.sync_memory_from_linear();
        Ok(())
    }
}

/// Extended context passed to compiled blocks that includes execution result
#[repr(C)]
pub struct ExtendedBlockContext {
    /// PVM registers (13 registers)
    pub registers: [u64; 13],
    /// Program counter  
    pub pc: u64,
    /// Pointer to linear memory buffer (for JIT access)
    pub memory_ptr: *mut u8,
    /// The execution result to be set by the block
    pub result: BlockExecutionResult,
}

/// Block-based JIT compiler for PVM programs
pub struct JitCompiler {
    /// Cache of compiled blocks indexed by starting PC
    block_cache: HashMap<u64, CompiledBlock>,
    /// Map of basic blocks discovered in the program
    basic_blocks: HashMap<u64, BasicBlock>,
    /// Jump table from the program (for dynamic jumps)
    jump_table: Vec<u64>,
    /// Original program bytes
    program_bytes: Vec<u8>,
    /// Cranelift compiler context
    compiler_context: cranelift_codegen::Context,
    /// Target ISA
    isa: cranelift_codegen::isa::OwnedTargetIsa,
}

impl JitCompiler {
    /// Create a new block-based JIT compiler
    pub fn new() -> Result<Self> {
        // Create target ISA for the current platform
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
            block_cache: HashMap::new(),
            basic_blocks: HashMap::new(),
            jump_table: Vec::new(),
            program_bytes: Vec::new(),
            compiler_context: cranelift_codegen::Context::new(),
            isa,
        })
    }

    /// Compile PVM program using block-based JIT
    pub fn compile(&mut self, program: &[u8]) -> Result<Module> {
        let has_explicit_trap = self.check_for_explicit_trap(program)?;
        Ok(Module::new(std::ptr::null(), 0, program.len(), has_explicit_trap)
            .with_program(program.to_vec()))
    }

    /// Analyze program and discover basic block boundaries
    pub fn analyze_program(&mut self, program: &[u8]) -> Result<()> {
        self.program_bytes = program.to_vec();
        
        // Parse program blob and extract jump table
        let blob = parser::program::deblob(program)?;
        self.jump_table = blob.jump_table.clone();
        
        // Discover basic block boundaries
        self.discover_basic_blocks(&blob)?;
        
        Ok(())
    }
    
    /// Execute program using block-based JIT compilation
    pub fn execute(&mut self, mut context: BlockContext) -> Result<Info> {
        loop {
            // Get or compile the block for current PC
            let block = self.get_or_compile_block(context.pc)?;
            
            // Execute the compiled block and get result
            let result = self.execute_block(&block, &mut context)?;
            
            // Handle control flow based on block execution result
            match result {
                BlockExecutionResult::Continue => {
                    // Find the next sequential block
                    if let Some(next_block_pc) = self.find_next_block_pc(context.pc) {
                        context.pc = next_block_pc;
                    } else {
                        // No next block - program ends
                        break;
                    }
                }
                BlockExecutionResult::Jump(target_pc) => {
                    // Jump to the specified PC
                    context.pc = target_pc;
                }
                BlockExecutionResult::Halt => {
                    // Normal program termination
                    break;
                }
                BlockExecutionResult::Trap => {
                    // Trap/panic - could set error state or break
                    break;
                }
            }
        }
        
        Ok(Info {
            registers: context.registers,
            pc: context.pc,
            memory: context.memory,
        })
    }

    /// Discover basic block boundaries using the official PVM library algorithm
    /// This implements the Graypaper formula: π ≡ ({0} ∪ {n + 1 + skip(n) | n ∈ N_{|c|} ∧ k_n = 1 ∧ c_n ∈ T}) ∩ {n | k_n = 1 ∧ c_n ∈ U}
    fn discover_basic_blocks(&mut self, blob: &parser::program::ProgramBlob) -> Result<()> {
        use std::collections::BTreeSet;
        
        let mut block_boundaries = BTreeSet::new();
        let mut reader = blob.reader();
        
        // Program always starts at offset 0 (part of the formula: {0} ∪ ...)
        block_boundaries.insert(0);
        
        // Use the PVM library's block reading logic to discover all block boundaries
        // This implements the official Graypaper basic-block sequence formula
        while !reader.eof() {
            let start_pc = reader.position;
            
            // Read a complete block using the official PVM library algorithm
            let block_instructions = reader.read_block()?;
            
            if !block_instructions.is_empty() {
                let last_instruction = &block_instructions[block_instructions.len() - 1];
                let block_end_pc = last_instruction.range.end;
                
                // Add the start of this block
                block_boundaries.insert(start_pc);
                
                // For branches and jumps, add their targets based on offsets
                match &last_instruction.value {
                    // Direct jumps and branches - add branch targets
                    Instruction::Jump(format) => {
                        let target = (last_instruction.range.start as i64 + format.off0 as i64) as usize;
                        if target < self.program_bytes.len() {
                            block_boundaries.insert(target);
                        }
                    }
                    Instruction::LoadImmJump(format) => {
                        let target = (last_instruction.range.start as i64 + format.off0 as i64) as usize;
                        if target < self.program_bytes.len() {
                            block_boundaries.insert(target);
                        }
                    }
                    Instruction::BranchEq(format) | Instruction::BranchNe(format) 
                    | Instruction::BranchLtU(format) | Instruction::BranchLtS(format)
                    | Instruction::BranchGeU(format) | Instruction::BranchGeS(format) => {
                        let target = (last_instruction.range.start as i64 + format.off0 as i64) as usize;
                        if target < self.program_bytes.len() {
                            block_boundaries.insert(target);
                        }
                    }
                    Instruction::BranchEqImm(format) | Instruction::BranchNeImm(format)
                    | Instruction::BranchLtUImm(format) | Instruction::BranchLtSImm(format)
                    | Instruction::BranchGeUImm(format) | Instruction::BranchGeSImm(format)
                    | Instruction::BranchLeUImm(format) | Instruction::BranchLeSImm(format)
                    | Instruction::BranchGtUImm(format) | Instruction::BranchGtSImm(format) => {
                        let target = (last_instruction.range.start as i64 + format.off0 as i64) as usize;
                        if target < self.program_bytes.len() {
                            block_boundaries.insert(target);
                        }
                    }
                    // Indirect jumps - add all jump table targets
                    Instruction::JumpInd(_) | Instruction::LoadImmJumpInd(_) => {
                        for &target in &self.jump_table {
                            if (target as usize) < self.program_bytes.len() {
                                block_boundaries.insert(target as usize);
                            }
                        }
                    }
                    _ => {}
                }
                
                // If there's a next instruction after this block, add it as a boundary
                if block_end_pc < self.program_bytes.len() {
                    block_boundaries.insert(block_end_pc);
                }
            }
        }
        
        // Convert boundaries to basic blocks
        let boundaries: Vec<_> = block_boundaries.into_iter().collect();
        for i in 0..boundaries.len() {
            let start_pc = boundaries[i];
            let end_pc = if i + 1 < boundaries.len() {
                boundaries[i + 1]
            } else {
                // Last block extends to end of program
                self.program_bytes.len()
            };
            
            // Skip empty blocks
            if start_pc >= end_pc {
                continue;
            }
            
            // Use the official PVM library logic to determine if block is terminating
            let is_terminating = self.is_block_terminating_official(start_pc, end_pc)?;
            
            self.basic_blocks.insert(start_pc as u64, BasicBlock {
                start_pc,
                end_pc,
                is_terminating,
                compiled_code: None,
            });
        }
        
        Ok(())
    }
    
    /// Dynamically discover a new basic block starting at the given PC
    fn discover_dynamic_block(&mut self, start_pc: u64) -> Result<BasicBlock> {
        let start_pc_usize = start_pc as usize;
        
        // Check if the PC is within program bounds
        if start_pc_usize >= self.program_bytes.len() {
            anyhow::bail!("Dynamic block discovery: PC {} is outside program bounds", start_pc);
        }
        
        // Use the official PVM library to discover this block
        let blob = parser::program::deblob(&self.program_bytes)?;
        let mut reader = blob.reader();
        reader.set_position(start_pc_usize);
        
        if reader.eof() {
            anyhow::bail!("Dynamic block discovery: PC {} is at end of program", start_pc);
        }
        
        // Read a complete block using the official algorithm
        let block_instructions = reader.read_block()?;
        
        if block_instructions.is_empty() {
            anyhow::bail!("Dynamic block discovery: No instructions found at PC {}", start_pc);
        }
        
        // Calculate block end PC
        let last_instruction = &block_instructions[block_instructions.len() - 1];
        let end_pc = last_instruction.range.end;
        
        // Check if block is terminating
        let is_terminating = self.is_block_terminating_official(start_pc_usize, end_pc)?;
        
        // Create the new basic block
        let basic_block = BasicBlock {
            start_pc: start_pc_usize,
            end_pc,
            is_terminating,
            compiled_code: None,
        };
        
        // Cache it for future use
        self.basic_blocks.insert(start_pc, basic_block.clone());
        
        tracing::debug!("Dynamically discovered block PC {}..{}", start_pc_usize, end_pc);
        Ok(basic_block)
    }

    /// Check if a block is terminating using the official PVM library logic
    /// This matches the terminating instruction list from the Graypaper and reader.rs
    fn is_block_terminating_official(&self, start_pc: usize, end_pc: usize) -> Result<bool> {
        if start_pc >= end_pc {
            return Ok(false);
        }
        
        let blob = parser::program::deblob(&self.program_bytes)?;
        let mut reader = blob.reader();
        reader.set_position(start_pc);
        
        // Read the block using the official PVM library block reading logic
        let block_instructions = reader.read_block()?;
        
        if block_instructions.is_empty() {
            return Ok(false);
        }
        
        // The block is terminating if the last instruction is a terminating instruction
        // This matches the official list from parser/reader.rs lines 79-100
        let last_instruction = &block_instructions[block_instructions.len() - 1];
        match &last_instruction.value {
            Instruction::Trap
            | Instruction::Fallthrough
            | Instruction::Jump(_)
            | Instruction::JumpInd(_)
            | Instruction::LoadImmJump(_)
            | Instruction::LoadImmJumpInd(_)
            | Instruction::BranchEq(_)
            | Instruction::BranchNe(_)
            | Instruction::BranchGeU(_)
            | Instruction::BranchGeS(_)
            | Instruction::BranchLtU(_)
            | Instruction::BranchLtS(_)
            | Instruction::BranchEqImm(_)
            | Instruction::BranchNeImm(_)
            | Instruction::BranchGeUImm(_)
            | Instruction::BranchGeSImm(_)
            | Instruction::BranchLtUImm(_)
            | Instruction::BranchLtSImm(_)
            | Instruction::BranchLeUImm(_)
            | Instruction::BranchLeSImm(_)
            | Instruction::BranchGtUImm(_)
            | Instruction::BranchGtSImm(_) => Ok(true),
            _ => Ok(false),
        }
    }

    /// Get or compile a basic block for the given PC
    fn get_or_compile_block(&mut self, pc: u64) -> Result<CompiledBlock> {
        if let Some(compiled) = self.block_cache.get(&pc).cloned() {
            return Ok(compiled);
        }
        
        // Compile the block
        let compiled = self.compile_block(pc)?;
        self.block_cache.insert(pc, compiled.clone());
        Ok(compiled)
    }
    
    /// Compile a single basic block
    pub fn compile_block(&mut self, start_pc: u64) -> Result<CompiledBlock> {
        // Check if we have a pre-discovered basic block
        let basic_block = if let Some(block) = self.basic_blocks.get(&start_pc) {
            block.clone()
        } else {
            // Dynamic block discovery: create a new block on-demand
            self.discover_dynamic_block(start_pc)?
        };
            
        // Create function signature for block execution
        // Block function signature: fn(*mut ExtendedBlockContext)
        let mut sig = Signature::new(self.isa.default_call_conv());
        sig.params.push(AbiParam::new(types::I64)); // pointer to ExtendedBlockContext
        
        // Create function and builder
        let mut func = Function::with_name_signature(UserFuncName::user(0, start_pc as u32), sig);
        let mut builder_context = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut func, &mut builder_context);
        
        // Create entry block
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);
        
        // Get the BlockContext pointer parameter
        let context_ptr = builder.block_params(entry_block)[0];
        
        // Use the translator to compile the block
        let result = self.compile_block_with_translator(&mut builder, context_ptr, &basic_block);
        
        match result {
            Ok(_) => {
                // Finalize the function
                builder.ins().return_(&[]);
                builder.finalize();
                
                // Compile the function
                self.compiler_context.clear();
                self.compiler_context.func = func;
                let mut ctrl_plane = cranelift_codegen::control::ControlPlane::default();
                self.compiler_context
                    .compile(&*self.isa, &mut ctrl_plane)
                    .map_err(|e| anyhow::anyhow!("Cranelift compilation failed: {:?}", e))?;
                
                // Get the compiled machine code
                let code = self.compiler_context.compiled_code().unwrap();
                let code_bytes = code.buffer.data();
                let code_size = code_bytes.len();
                
                // Allocate executable memory and copy code
                let executable_ptr = self.allocate_executable_memory(code_size)?;
                unsafe {
                    std::ptr::copy_nonoverlapping(code_bytes.as_ptr(), executable_ptr, code_size);
                }
                
                Ok(CompiledBlock {
                    entry_point: executable_ptr,
                    size: code_size,
                })
            }
            Err(e) => {
                // For now, if compilation fails, return a placeholder that does nothing
                tracing::warn!("Block compilation failed for PC {}: {}", start_pc, e);
                Ok(CompiledBlock {
                    entry_point: std::ptr::null(),
                    size: 0,
                })
            }
        }
    }
    
    /// Compile instructions for a single basic block using the Translator
    fn compile_block_with_translator(
        &self, 
        builder: &mut FunctionBuilder, 
        context_ptr: Value,
        basic_block: &BasicBlock
    ) -> Result<()> {
        // Create a translator that will handle all instruction compilation
        let mut translator = Translator::new(builder);
        
        // Load initial context (registers, PC, memory) from the context pointer
        translator.load_initial_context(context_ptr)?;
        
        // Use the translator's new translate_block method
        translator.translate_block(&self.program_bytes, basic_block.start_pc, basic_block.end_pc)?;
        
        // Save all state back to context before returning
        // Store all 13 register values back to context.registers
        for i in 0..13 {
            let reg_var = translator.registers[&(i as u8)];
            let reg_value = translator.builder.use_var(reg_var);
            let offset = translator.builder.ins().iconst(types::I64, (i * 8) as i64);
            let addr = translator.builder.ins().iadd(context_ptr, offset);
            translator.builder.ins().store(MemFlags::new(), reg_value, addr, 0);
        }
        
        // Store PC back to context.pc (offset 104)
        let pc_value = translator.builder.use_var(translator.pc);
        let pc_offset = translator.builder.ins().iconst(types::I64, 104);
        let pc_addr = translator.builder.ins().iadd(context_ptr, pc_offset);
        translator.builder.ins().store(MemFlags::new(), pc_value, pc_addr, 0);
        
        tracing::debug!("Compiled block PC {}..{} using translator", basic_block.start_pc, basic_block.end_pc);
        Ok(())
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
    
    /// Find the next sequential block PC after the current PC
    fn find_next_block_pc(&self, current_pc: u64) -> Option<u64> {
        // Find the current block
        let current_block = self.basic_blocks.get(&current_pc)?;
        
        // Look for a block that starts at the end of the current block
        self.basic_blocks.keys()
            .find(|&&pc| pc == current_block.end_pc as u64)
            .copied()
    }

    /// Execute a compiled block and return the execution result
    fn execute_block(&self, block: &CompiledBlock, context: &mut BlockContext) -> Result<BlockExecutionResult> {
        if block.entry_point.is_null() {
            // Block compilation failed or placeholder - continue to next block
            return Ok(BlockExecutionResult::Continue);
        }
        
        // Create an extended context with linear memory pointer
        let mut extended_context = ExtendedBlockContext {
            registers: context.registers,
            pc: context.pc,
            memory_ptr: context.linear_memory.as_mut_ptr(),
            result: BlockExecutionResult::Continue,
        };
        
        unsafe {
            // Call the compiled block function
            // Block function signature: fn(*mut ExtendedBlockContext)
            let func_ptr = std::mem::transmute::<*const u8, fn(*mut ExtendedBlockContext)>(block.entry_point);
            func_ptr(&mut extended_context);
        }
        
        // Copy back the register and PC changes
        context.registers = extended_context.registers;
        context.pc = extended_context.pc;
        
        // Read back the execution result from the context memory
        let execution_result = self.decode_execution_result(&extended_context)?;
        
        // Sync linear memory changes back to Memory structure and check for access violations
        let sync_result = context.sync_memory_from_linear_with_validation();
        match sync_result {
            Ok(_) => Ok(execution_result),
            Err(_) => {
                // Memory access violation detected - return trap result and set PC=0
                context.pc = 0;
                Ok(BlockExecutionResult::Trap)
            }
        }
    }
    
    /// Decode the execution result from the extended context memory
    fn decode_execution_result(&self, extended_context: &ExtendedBlockContext) -> Result<BlockExecutionResult> {
        // The result is stored at offset 120 in the ExtendedBlockContext
        // Layout: discriminant (8 bytes) + data (8 bytes)
        unsafe {
            let context_ptr = extended_context as *const ExtendedBlockContext as *const u8;
            let result_ptr = context_ptr.add(120); // offset 120 for result field
            
            // Read discriminant
            let discriminant = *(result_ptr as *const u64);
            
            match discriminant {
                0 => Ok(BlockExecutionResult::Continue),
                1 => {
                    // Jump variant - read target PC from offset +8
                    let target_pc = *(result_ptr.add(8) as *const u64);
                    Ok(BlockExecutionResult::Jump(target_pc))
                }
                2 => Ok(BlockExecutionResult::Halt),
                3 => Ok(BlockExecutionResult::Trap),
                _ => {
                    tracing::warn!("Unknown execution result discriminant: {}", discriminant);
                    Ok(BlockExecutionResult::Continue)
                }
            }
        }
    }

    /// Get discovered basic blocks (for testing)
    pub fn get_basic_blocks(&self) -> &HashMap<u64, BasicBlock> {
        &self.basic_blocks
    }

    /// Check if program contains explicit trap instructions
    fn check_for_explicit_trap(&self, program: &[u8]) -> Result<bool> {
        // Quick check for trap instruction
        let blob = parser::program::deblob(program)?;
        let mut reader = blob.reader();
        
        while !reader.eof() {
            let instruction_offset = reader.read()?;
            if matches!(instruction_offset.value, parser::Instruction::Trap) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
}