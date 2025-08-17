//! Clean block-based JIT compiler for PVM programs

use crate::{
    constants::{context_offsets, exec_result, JUMP_ALIGNMENT_FACTOR, PVM_REGISTER_COUNT},
    translator::Translator,
    utils, Info, Module,
};
use anyhow::Result;
use cranelift::prelude::*;
use cranelift_codegen::ir::{Function, UserFuncName};
use parser::Visitor;
use std::collections::HashMap;
pub use {
    context::{Context, ExtendedContext},
    result::ExecResult,
};

mod context;
mod result;

/// JIT compiler
pub struct Jit {
    /// Cache of compiled blocks
    code_cache: HashMap<u64, Code>,
    /// Map of blocks by start PC
    blocks: HashMap<u64, Block>,
    /// Jump table for indirect jumps
    jump_table: Vec<u64>,
    /// Program bytes
    program: Vec<u8>,
    /// PC at the end of the entire program
    program_end_pc: u64,
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
                    || utils::is_terminating_instruction(
                        &block_instructions.last().unwrap().value,
                    ));

            // Handle indirect jump table targets first
            self.process_jump_targets(&block_instructions, &blob)?;

            // Only create block if it doesn't already exist (might have been created by process_jump_targets)
            if !self.blocks.contains_key(&(block_start as u64)) {
                self.create_block(block_start, reader.position, terminates, block_instructions);
            }
        }

        // Store the end PC of the last instruction in the entire program
        self.program_end_pc = last_instruction_pc;
        tracing::trace!("PROGRAM END PC set to: {}", self.program_end_pc);

        Ok(())
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
        tracing::debug!("Starting unified execution with initial PC: {}", ctx.pc);

        // Use unified compilation - execute entire program in one function call
        let code = self.get_code(ctx.pc)?;
        self.run_unified(&code, &mut ctx)?;

        Ok(Info {
            registers: ctx.registers,
            pc: ctx.pc,
            memory: ctx.memory.clone(),
        })
    }

    // Old block-based execution loop - replaced by unified compilation
    #[allow(dead_code)]
    fn execute_block_based(&mut self, mut ctx: Context) -> Result<Info> {
        loop {
            tracing::debug!("Executing block at PC: {}", ctx.pc);
            let code = self.get_code(ctx.pc)?;
            let block = self.blocks.get(&ctx.pc);
            let (result, _pc_managed) = self.run_block(&code, &mut ctx, block)?;

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

    /// Compile entire program as unified Cranelift function (cranelift-wasm style)
    fn compile_unified_program(&mut self) -> Result<Code> {
        tracing::debug!("Compiling entire program as unified Cranelift function");

        // Create function signature: takes context pointer AND starting PC, returns void
        let mut sig = Signature::new(self.isa.default_call_conv());
        sig.params.push(AbiParam::new(types::I64)); // context pointer
        sig.params.push(AbiParam::new(types::I64)); // starting PC

        let mut func = Function::with_name_signature(UserFuncName::user(0, 0), sig);
        let mut builder_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut func, &mut builder_ctx);

        // Create entry block
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);

        let ctx_ptr = builder.block_params(entry)[0];
        let start_pc = builder.block_params(entry)[1];

        // Step 1: Create all Cranelift blocks upfront (enables forward jumps)
        let mut block_map = std::collections::HashMap::new();
        for &pc in self.blocks.keys() {
            let cranelift_block = builder.create_block();
            block_map.insert(pc, cranelift_block);
            tracing::trace!("Created Cranelift block for PVM PC {}", pc);
        }

        // Use scoped translator to avoid ownership issues
        {
            let mut translator = Translator::new(&mut builder);
            translator.init_with_context(ctx_ptr)?;
            translator.set_unified_mode(true);

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
            for (&pc, &cranelift_block) in &block_map {
                switch.set_entry(pc as u128, cranelift_block);
            }

            // Default case: if PC is not found, return with trap
            let default_block = translator.builder.create_block();
            translator.builder.switch_to_block(default_block);
            self.return_trap(&mut translator)?;
            translator.builder.seal_block(default_block);

            // Generate the switch on start_pc
            translator.builder.switch_to_block(entry);
            switch.emit(translator.builder, start_pc, default_block);
            translator.builder.seal_block(entry);

            // Step 2: Translate all PVM blocks to Cranelift basic blocks using shared translator
            for (&pc, pvm_block) in &self.blocks {
                let cranelift_block = block_map[&pc];
                translator.builder.switch_to_block(cranelift_block);

                tracing::trace!("Translating PVM block at PC {} to Cranelift block", pc);

                // Translate instructions in this block using shared translator
                match self.translate_unified_block_with_translator(
                    &mut translator,
                    pvm_block,
                    &block_map,
                ) {
                    Ok(_) => {
                        tracing::trace!("Successfully translated block at PC {}", pc);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to translate block at PC {}: {}", pc, e);
                        // Generate trap for failed blocks
                        self.return_trap(&mut translator)?;
                    }
                }
            }

            // Step 3: Seal all blocks after translation
            for &cranelift_block in block_map.values() {
                translator.builder.seal_block(cranelift_block);
            }
        } // translator goes out of scope here

        // Finalize the function
        builder.finalize();

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

    /// Compile single basic block (legacy method for compatibility)
    fn compile_block(&mut self, pc: u64) -> Result<Code> {
        // For unified compilation, we compile the whole program once and cache it
        if self.code_cache.is_empty() {
            let unified_code = self.compile_unified_program()?;
            // Cache the unified code for all PCs
            for &block_pc in self.blocks.keys() {
                self.code_cache.insert(block_pc, unified_code.clone());
            }
        }

        // Return the cached unified code
        self.code_cache
            .get(&pc)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No unified code for PC {}", pc))
    }

    /// Translate block for unified function (cranelift-wasm style)
    #[allow(dead_code)]
    fn translate_unified_block(
        &self,
        builder: &mut FunctionBuilder,
        ctx_ptr: Value,
        block: &Block,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        let mut translator = Translator::new(builder);
        translator.init_with_context(ctx_ptr)?;
        translator.set_unified_mode(true); // Enable unified mode to suppress visitor terminators

        // In unified compilation, registers are loaded ONCE at function entry
        // and stay in Cranelift variables throughout the entire function
        // No need to load/save per block

        // Translate all instructions in this block
        for instruction in &block.instructions {
            let pc = instruction.range.start;
            tracing::trace!(
                "Unified: translating PC {} instruction {:?}",
                pc,
                instruction.value
            );

            if let Err(e) = translator.visit(instruction.value, pc) {
                tracing::warn!("Instruction translation failed at PC {}: {}", pc, e);
                // Continue with best effort
            }
        }

        // Handle block termination with native Cranelift control flow
        self.handle_unified_block_termination(&mut translator, block, block_map)?;

        Ok(())
    }

    /// Translate block for unified function using shared translator instance
    fn translate_unified_block_with_translator(
        &self,
        translator: &mut Translator,
        block: &Block,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        // Translate all instructions in this block
        for instruction in &block.instructions {
            let pc = instruction.range.start;
            tracing::trace!(
                "Unified: translating PC {} instruction {:?}",
                pc,
                instruction.value
            );

            if let Err(e) = translator.visit(instruction.value, pc) {
                tracing::warn!("Instruction translation failed at PC {}: {}", pc, e);
                // Continue with best effort
            }
        }

        // Handle block termination with native Cranelift control flow
        self.handle_unified_block_termination_with_translator(translator, block, block_map)?;

        Ok(())
    }

    /// Handle block termination using native Cranelift control flow
    #[allow(dead_code)]
    fn handle_unified_block_termination(
        &self,
        translator: &mut Translator,
        block: &Block,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        if let Some(last_instruction) = block.instructions.last() {
            let pc = last_instruction.range.start;

            match &last_instruction.value {
                // Handle jumps with native Cranelift control flow
                parser::Instruction::Jump(format) => {
                    let target_pc = (pc as i64 + format.off0 as i64) as u64;
                    if let Some(&target_block) = block_map.get(&target_pc) {
                        translator.builder.ins().jump(target_block, &[]);
                    } else {
                        // Jump to unknown target - return with jump result
                        self.return_with_jump_result(translator, target_pc)?;
                    }
                }

                // Handle branches with native Cranelift control flow
                parser::Instruction::BranchEq(_)
                | parser::Instruction::BranchNe(_)
                | parser::Instruction::BranchEqImm(_)
                | parser::Instruction::BranchNeImm(_) => {
                    // Branch instructions should set up their own control flow in visitor
                    // If we reach here, it means branch wasn't taken - fall through to next block
                    let next_pc = last_instruction.range.end as u64;
                    if let Some(&next_block) = block_map.get(&next_pc) {
                        translator.builder.ins().jump(next_block, &[]);
                    } else {
                        // No next block - program ends
                        self.return_continue(translator)?;
                    }
                }

                // Handle indirect jumps with runtime dispatch
                parser::Instruction::JumpInd(_) | parser::Instruction::LoadImmJumpInd(_) => {
                    // In unified mode, generate a runtime switch to dispatch to the target block
                    self.handle_indirect_jump_unified(translator, pc, block_map)?;
                }

                // Handle traps and halts
                parser::Instruction::Trap => {
                    self.return_trap_with_pc(translator, pc)?;
                }

                _ => {
                    // Non-terminating instruction - fall through to next block
                    let next_pc = last_instruction.range.end as u64;
                    if let Some(&next_block) = block_map.get(&next_pc) {
                        translator.builder.ins().jump(next_block, &[]);
                    } else {
                        // No next block - program ends at the end of this instruction
                        self.return_continue_with_pc(translator, next_pc)?;
                    }
                }
            }
        } else {
            // Empty block - continue
            self.return_continue(translator)?;
        }

        Ok(())
    }

    /// Handle block termination using native Cranelift control flow with shared translator
    fn handle_unified_block_termination_with_translator(
        &self,
        translator: &mut Translator,
        block: &Block,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        if let Some(last_instruction) = block.instructions.last() {
            let pc = last_instruction.range.start;

            match &last_instruction.value {
                // Handle jumps with native Cranelift control flow
                parser::Instruction::Jump(format) => {
                    let target_pc = (pc as i64 + format.off0 as i64) as u64;
                    if let Some(&target_block) = block_map.get(&target_pc) {
                        translator.builder.ins().jump(target_block, &[]);
                    } else {
                        // Jump to unknown target - return with jump result
                        self.return_with_jump_result(translator, target_pc)?;
                    }
                }

                // Handle LoadImmJump with native Cranelift control flow
                parser::Instruction::LoadImmJump(format) => {
                    let target_pc = (pc as i64 + format.off0 as i64) as u64;
                    if let Some(&target_block) = block_map.get(&target_pc) {
                        translator.builder.ins().jump(target_block, &[]);
                    } else {
                        // Jump to unknown target - return with jump result
                        self.return_with_jump_result(translator, target_pc)?;
                    }
                }

                // Handle branches with native Cranelift control flow
                parser::Instruction::BranchEq(format) => {
                    self.handle_branch_eq_unified(
                        translator,
                        format,
                        pc,
                        last_instruction.range.end,
                        block_map,
                    )?;
                }
                parser::Instruction::BranchNe(format) => {
                    self.handle_branch_ne_unified(
                        translator,
                        format,
                        pc,
                        last_instruction.range.end,
                        block_map,
                    )?;
                }
                parser::Instruction::BranchEqImm(format) => {
                    self.handle_branch_eq_imm_unified(
                        translator,
                        format,
                        pc,
                        last_instruction.range.end,
                        block_map,
                    )?;
                }
                parser::Instruction::BranchNeImm(format) => {
                    self.handle_branch_ne_imm_unified(
                        translator,
                        format,
                        pc,
                        last_instruction.range.end,
                        block_map,
                    )?;
                }
                // Add other branch variants as needed
                parser::Instruction::BranchLtU(format) => {
                    self.handle_branch_lt_u_unified(
                        translator,
                        format,
                        pc,
                        last_instruction.range.end,
                        block_map,
                    )?;
                }
                parser::Instruction::BranchLtS(format) => {
                    self.handle_branch_lt_s_unified(
                        translator,
                        format,
                        pc,
                        last_instruction.range.end,
                        block_map,
                    )?;
                }
                parser::Instruction::BranchGeU(format) => {
                    self.handle_branch_ge_u_unified(
                        translator,
                        format,
                        pc,
                        last_instruction.range.end,
                        block_map,
                    )?;
                }
                parser::Instruction::BranchGeS(format) => {
                    self.handle_branch_ge_s_unified(
                        translator,
                        format,
                        pc,
                        last_instruction.range.end,
                        block_map,
                    )?;
                }
                // Handle immediate variants
                parser::Instruction::BranchLtUImm(format) => {
                    self.handle_branch_lt_u_imm_unified(
                        translator,
                        format,
                        pc,
                        last_instruction.range.end,
                        block_map,
                    )?;
                }
                parser::Instruction::BranchLtSImm(format) => {
                    self.handle_branch_lt_s_imm_unified(
                        translator,
                        format,
                        pc,
                        last_instruction.range.end,
                        block_map,
                    )?;
                }
                parser::Instruction::BranchGeUImm(format) => {
                    self.handle_branch_ge_u_imm_unified(
                        translator,
                        format,
                        pc,
                        last_instruction.range.end,
                        block_map,
                    )?;
                }
                parser::Instruction::BranchGeSImm(format) => {
                    self.handle_branch_ge_s_imm_unified(
                        translator,
                        format,
                        pc,
                        last_instruction.range.end,
                        block_map,
                    )?;
                }
                parser::Instruction::BranchGtUImm(format) => {
                    self.handle_branch_gt_u_imm_unified(
                        translator,
                        format,
                        pc,
                        last_instruction.range.end,
                        block_map,
                    )?;
                }
                parser::Instruction::BranchGtSImm(format) => {
                    self.handle_branch_gt_s_imm_unified(
                        translator,
                        format,
                        pc,
                        last_instruction.range.end,
                        block_map,
                    )?;
                }
                parser::Instruction::BranchLeUImm(format) => {
                    self.handle_branch_le_u_imm_unified(
                        translator,
                        format,
                        pc,
                        last_instruction.range.end,
                        block_map,
                    )?;
                }
                parser::Instruction::BranchLeSImm(format) => {
                    self.handle_branch_le_s_imm_unified(
                        translator,
                        format,
                        pc,
                        last_instruction.range.end,
                        block_map,
                    )?;
                }

                // Handle indirect jumps with runtime dispatch
                parser::Instruction::JumpInd(_) | parser::Instruction::LoadImmJumpInd(_) => {
                    // In unified mode, generate a runtime switch to dispatch to the target block
                    self.handle_indirect_jump_unified(translator, pc, block_map)?;
                }

                // Handle traps and halts
                parser::Instruction::Trap => {
                    self.return_trap_with_pc(translator, pc)?;
                }

                _ => {
                    // Non-terminating instruction - fall through to next block
                    let next_pc = last_instruction.range.end as u64;
                    if let Some(&next_block) = block_map.get(&next_pc) {
                        translator.builder.ins().jump(next_block, &[]);
                    } else {
                        // No next block - program ends at the end of this instruction
                        self.return_continue_with_pc(translator, next_pc)?;
                    }
                }
            }
        } else {
            // Empty block - continue
            self.return_continue(translator)?;
        }

        Ok(())
    }

    /// Save all registers from Cranelift variables back to context
    fn save_registers_to_context(&self, translator: &mut Translator) -> Result<()> {
        let ctx_ptr = translator
            .get_context_ptr()
            .expect("Context pointer not initialized");

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

    /// Return with continue result (used by empty blocks)
    fn return_continue(&self, translator: &mut Translator) -> Result<()> {
        // For empty blocks, use PC 0 as we don't have a specific PC
        self.return_continue_with_pc(translator, 0)
    }

    /// Return with continue result and specific PC
    fn return_continue_with_pc(&self, translator: &mut Translator, pc: u64) -> Result<()> {
        let ctx_ptr = translator
            .get_context_ptr()
            .expect("Context pointer not initialized");

        // Save all registers back to context before returning
        self.save_registers_to_context(translator)?;

        // Save the PC
        let pc_offset = translator
            .builder
            .ins()
            .iconst(types::I64, context_offsets::PC_OFFSET as i64);
        let pc_addr = translator.builder.ins().iadd(ctx_ptr, pc_offset);
        let pc_val = translator.builder.ins().iconst(types::I64, pc as i64);
        translator
            .builder
            .ins()
            .store(MemFlags::new(), pc_val, pc_addr, 0);

        let result_offset = translator
            .builder
            .ins()
            .iconst(types::I64, context_offsets::RESULT_OFFSET as i64);
        let result_addr = translator.builder.ins().iadd(ctx_ptr, result_offset);
        let continue_discriminant = translator
            .builder
            .ins()
            .iconst(types::I64, exec_result::CONTINUE as i64);
        translator
            .builder
            .ins()
            .store(MemFlags::new(), continue_discriminant, result_addr, 0);
        translator.builder.ins().return_(&[]);
        Ok(())
    }

    /// Return with jump result
    fn return_with_jump_result(&self, translator: &mut Translator, target_pc: u64) -> Result<()> {
        let ctx_ptr = translator
            .get_context_ptr()
            .expect("Context pointer not initialized");

        // Save all registers back to context before returning
        self.save_registers_to_context(translator)?;

        let result_offset = translator
            .builder
            .ins()
            .iconst(types::I64, context_offsets::RESULT_OFFSET as i64);
        let result_addr = translator.builder.ins().iadd(ctx_ptr, result_offset);

        let jump_discriminant = translator
            .builder
            .ins()
            .iconst(types::I64, exec_result::JUMP as i64);
        translator
            .builder
            .ins()
            .store(MemFlags::new(), jump_discriminant, result_addr, 0);

        let data_offset = translator.builder.ins().iconst(types::I64, 8);
        let data_addr = translator.builder.ins().iadd(result_addr, data_offset);
        let target_val = translator
            .builder
            .ins()
            .iconst(types::I64, target_pc as i64);
        translator
            .builder
            .ins()
            .store(MemFlags::new(), target_val, data_addr, 0);

        translator.builder.ins().return_(&[]);
        Ok(())
    }

    /// Handle indirect jump in unified mode - generate runtime dispatch with proper validation
    fn handle_indirect_jump_unified(
        &self,
        translator: &mut Translator,
        instruction_pc: usize,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        let ctx_ptr = translator
            .get_context_ptr()
            .expect("Context pointer not initialized");

        // Read the target address that was computed and stored by the visitor
        let result_offset = translator
            .builder
            .ins()
            .iconst(types::I64, context_offsets::RESULT_OFFSET as i64);
        let result_addr = translator.builder.ins().iadd(ctx_ptr, result_offset);
        let data_offset = translator.builder.ins().iconst(types::I64, 8);
        let data_addr = translator.builder.ins().iadd(result_addr, data_offset);
        let target_addr = translator
            .builder
            .ins()
            .load(types::I64, MemFlags::new(), data_addr, 0);

        // Implement the same validation logic as the interpreter:
        // 1. address == 0 (null address)
        // 2. address > table.len() * JUMP_ALIGNMENT_FACTOR (beyond table bounds)
        // 3. address % 2 != 0 (not aligned to 2-byte boundary)

        let _current_block = translator.builder.current_block().unwrap();
        let valid_jump_block = translator.builder.create_block();
        let trap_block = translator.builder.create_block();

        // Check 1: address == 0
        let zero = translator.builder.ins().iconst(types::I64, 0);
        let is_zero = translator
            .builder
            .ins()
            .icmp(IntCC::Equal, target_addr, zero);

        // Check 2: address > table.len() * JUMP_ALIGNMENT_FACTOR
        let table_len = self.jump_table.len() as u32;
        let jump_alignment_factor = JUMP_ALIGNMENT_FACTOR;
        let max_address = table_len * jump_alignment_factor;
        let max_addr_val = translator
            .builder
            .ins()
            .iconst(types::I64, max_address as i64);
        let exceeds_bounds =
            translator
                .builder
                .ins()
                .icmp(IntCC::UnsignedGreaterThan, target_addr, max_addr_val);

        // Check 3: address % 2 != 0 (misaligned)
        let two = translator.builder.ins().iconst(types::I64, 2);
        let remainder = translator.builder.ins().urem(target_addr, two);
        let is_misaligned = translator
            .builder
            .ins()
            .icmp(IntCC::NotEqual, remainder, zero);

        // Combine all invalid conditions with OR
        let invalid1 = translator.builder.ins().bor(is_zero, exceeds_bounds);
        let invalid_jump = translator.builder.ins().bor(invalid1, is_misaligned);

        // Branch: if invalid, trap; otherwise continue to valid jump handling
        translator
            .builder
            .ins()
            .brif(invalid_jump, trap_block, &[], valid_jump_block, &[]);

        // Valid jump block: calculate index and dispatch
        translator.builder.switch_to_block(valid_jump_block);

        // Calculate jump table index: (address / 2) - 1 (following interpreter logic)
        let addr_div_2 = translator.builder.ins().udiv(target_addr, two);
        let one = translator.builder.ins().iconst(types::I64, 1);
        let jump_index = translator.builder.ins().isub(addr_div_2, one);

        // Create switch to dispatch to correct block based on jump table index
        let mut switch = cranelift::frontend::Switch::new();

        // Add all jump table entries as switch cases
        for (i, &jump_pc) in self.jump_table.iter().enumerate() {
            if let Some(&cranelift_block) = block_map.get(&jump_pc) {
                switch.set_entry(i as u128, cranelift_block);
            }
        }

        // Emit the switch (default case goes to trap for out-of-bounds indices)
        switch.emit(translator.builder, jump_index, trap_block);

        // Trap block: invalid jump target
        translator.builder.switch_to_block(trap_block);
        self.return_trap_with_pc(translator, instruction_pc)?;

        // Seal all created blocks
        translator.builder.seal_block(valid_jump_block);
        translator.builder.seal_block(trap_block);

        Ok(())
    }

    /// Return with indirect jump result (no longer used in unified mode)
    #[allow(dead_code)]
    fn return_with_indirect_jump(
        &self,
        translator: &mut Translator,
        instruction_pc: usize,
    ) -> Result<()> {
        let ctx_ptr = translator
            .get_context_ptr()
            .expect("Context pointer not initialized");

        // Save all registers back to context before returning
        self.save_registers_to_context(translator)?;

        // Set PC to the instruction that caused the indirect jump
        translator.store_instruction_pc(ctx_ptr, instruction_pc)?;

        let result_offset = translator
            .builder
            .ins()
            .iconst(types::I64, context_offsets::RESULT_OFFSET as i64);
        let result_addr = translator.builder.ins().iadd(ctx_ptr, result_offset);
        let indirect_discriminant = translator
            .builder
            .ins()
            .iconst(types::I64, exec_result::JUMP_INDIRECT as i64);
        translator
            .builder
            .ins()
            .store(MemFlags::new(), indirect_discriminant, result_addr, 0);
        translator.builder.ins().return_(&[]);
        Ok(())
    }

    /// Return with trap result
    fn return_trap(&self, translator: &mut Translator) -> Result<()> {
        let ctx_ptr = translator
            .get_context_ptr()
            .expect("Context pointer not initialized");

        // Save all registers back to context before returning
        self.save_registers_to_context(translator)?;

        let result_offset = translator
            .builder
            .ins()
            .iconst(types::I64, context_offsets::RESULT_OFFSET as i64);
        let result_addr = translator.builder.ins().iadd(ctx_ptr, result_offset);
        let trap_discriminant = translator
            .builder
            .ins()
            .iconst(types::I64, exec_result::TRAP as i64);
        translator
            .builder
            .ins()
            .store(MemFlags::new(), trap_discriminant, result_addr, 0);
        translator.builder.ins().return_(&[]);
        Ok(())
    }

    /// Return with trap result and set PC to the trap instruction location
    fn return_trap_with_pc(&self, translator: &mut Translator, trap_pc: usize) -> Result<()> {
        let ctx_ptr = translator
            .get_context_ptr()
            .expect("Context pointer not initialized");

        // Save all registers back to context before returning
        self.save_registers_to_context(translator)?;

        // Set PC to the trap instruction location
        let pc_offset = translator
            .builder
            .ins()
            .iconst(types::I64, context_offsets::PC_OFFSET as i64);
        let pc_addr = translator.builder.ins().iadd(ctx_ptr, pc_offset);
        let pc_val = translator.builder.ins().iconst(types::I64, trap_pc as i64);
        translator
            .builder
            .ins()
            .store(MemFlags::new(), pc_val, pc_addr, 0);

        let result_offset = translator
            .builder
            .ins()
            .iconst(types::I64, context_offsets::RESULT_OFFSET as i64);
        let result_addr = translator.builder.ins().iadd(ctx_ptr, result_offset);
        let trap_discriminant = translator
            .builder
            .ins()
            .iconst(types::I64, exec_result::TRAP as i64);
        translator
            .builder
            .ins()
            .store(MemFlags::new(), trap_discriminant, result_addr, 0);
        translator.builder.ins().return_(&[]);
        Ok(())
    }

    /// Translate block using Translator (legacy method)
    #[allow(dead_code)]
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

        translator.translate_block_with_instructions(&self.program, block)?;

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
    /// Run the unified compiled function - executes entire program
    fn run_unified(&self, code: &Code, ctx: &mut Context) -> Result<()> {
        if code.ptr.is_null() {
            return Ok(());
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

    // Old block-based execution - kept for reference but not used
    #[allow(dead_code)]
    fn run_block(
        &self,
        code: &Code,
        ctx: &mut Context,
        _block: Option<&Block>,
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
            let func = std::mem::transmute::<*const u8, fn(*mut ExtendedContext, u64)>(code.ptr);
            func(&mut ext_ctx, ctx.pc);
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

    /// Handle BranchEq instruction in unified mode with native Cranelift control flow
    fn handle_branch_eq_unified(
        &self,
        translator: &mut Translator,
        format: &parser::format::RRO,
        pc: usize,
        next_pc: usize,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        let parser::format::RRO { reg0, reg1, off0 } = *format;

        // Compare registers
        let reg0_val = translator.builder.use_var(translator.registers[&reg0]);
        let reg1_val = translator.builder.use_var(translator.registers[&reg1]);
        let condition = translator
            .builder
            .ins()
            .icmp(IntCC::Equal, reg0_val, reg1_val);

        // Calculate target addresses
        let target_pc = (pc as i64 + off0 as i64) as u64;

        self.generate_unified_branch(translator, condition, target_pc, next_pc as u64, block_map)
    }

    /// Handle BranchNe instruction in unified mode
    fn handle_branch_ne_unified(
        &self,
        translator: &mut Translator,
        format: &parser::format::RRO,
        pc: usize,
        next_pc: usize,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        let parser::format::RRO { reg0, reg1, off0 } = *format;
        let reg0_val = translator.builder.use_var(translator.registers[&reg0]);
        let reg1_val = translator.builder.use_var(translator.registers[&reg1]);
        let condition = translator
            .builder
            .ins()
            .icmp(IntCC::NotEqual, reg0_val, reg1_val);

        let target_pc = (pc as i64 + off0 as i64) as u64;

        self.generate_unified_branch(translator, condition, target_pc, next_pc as u64, block_map)
    }

    /// Handle BranchEqImm instruction in unified mode
    fn handle_branch_eq_imm_unified(
        &self,
        translator: &mut Translator,
        format: &parser::format::RIO,
        pc: usize,
        next_pc: usize,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        let parser::format::RIO { reg0, imm0, off0 } = *format;
        let reg_val = translator.builder.use_var(translator.registers[&reg0]);
        let imm_val = translator.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = translator
            .builder
            .ins()
            .icmp(IntCC::Equal, reg_val, imm_val);

        let target_pc = (pc as i64 + off0 as i64) as u64;

        self.generate_unified_branch(translator, condition, target_pc, next_pc as u64, block_map)
    }

    /// Handle BranchNeImm instruction in unified mode
    fn handle_branch_ne_imm_unified(
        &self,
        translator: &mut Translator,
        format: &parser::format::RIO,
        pc: usize,
        next_pc: usize,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        let parser::format::RIO { reg0, imm0, off0 } = *format;
        let reg_val = translator.builder.use_var(translator.registers[&reg0]);
        let imm_val = translator.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = translator
            .builder
            .ins()
            .icmp(IntCC::NotEqual, reg_val, imm_val);

        let target_pc = (pc as i64 + off0 as i64) as u64;

        self.generate_unified_branch(translator, condition, target_pc, next_pc as u64, block_map)
    }

    /// Generic helper for unified branch generation
    fn generate_unified_branch(
        &self,
        translator: &mut Translator,
        condition: Value,
        target_pc: u64,
        next_pc: u64,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        if let (Some(&target_block), Some(&next_block)) =
            (block_map.get(&target_pc), block_map.get(&next_pc))
        {
            translator
                .builder
                .ins()
                .brif(condition, target_block, &[], next_block, &[]);
        } else if let Some(&target_block) = block_map.get(&target_pc) {
            let cont_block = translator.builder.create_block();
            translator
                .builder
                .ins()
                .brif(condition, target_block, &[], cont_block, &[]);
            translator.builder.switch_to_block(cont_block);
            self.return_continue(translator)?;
        } else if let Some(&next_block) = block_map.get(&next_pc) {
            let jump_block = translator.builder.create_block();
            translator
                .builder
                .ins()
                .brif(condition, jump_block, &[], next_block, &[]);
            translator.builder.switch_to_block(jump_block);
            self.return_with_jump_result(translator, target_pc)?;
        } else {
            let jump_block = translator.builder.create_block();
            let cont_block = translator.builder.create_block();
            translator
                .builder
                .ins()
                .brif(condition, jump_block, &[], cont_block, &[]);

            translator.builder.switch_to_block(jump_block);
            self.return_with_jump_result(translator, target_pc)?;

            translator.builder.switch_to_block(cont_block);
            self.return_continue(translator)?;
        }
        Ok(())
    }

    // Implementations for all branch types
    fn handle_branch_lt_u_unified(
        &self,
        translator: &mut Translator,
        format: &parser::format::RRO,
        pc: usize,
        next_pc: usize,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        let parser::format::RRO { reg0, reg1, off0 } = *format;
        let reg0_val = translator.builder.use_var(translator.registers[&reg0]);
        let reg1_val = translator.builder.use_var(translator.registers[&reg1]);
        let condition = translator
            .builder
            .ins()
            .icmp(IntCC::UnsignedLessThan, reg0_val, reg1_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(translator, condition, target_pc, next_pc as u64, block_map)
    }

    fn handle_branch_lt_s_unified(
        &self,
        translator: &mut Translator,
        format: &parser::format::RRO,
        pc: usize,
        next_pc: usize,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        let parser::format::RRO { reg0, reg1, off0 } = *format;
        let reg0_val = translator.builder.use_var(translator.registers[&reg0]);
        let reg1_val = translator.builder.use_var(translator.registers[&reg1]);
        let condition = translator
            .builder
            .ins()
            .icmp(IntCC::SignedLessThan, reg0_val, reg1_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(translator, condition, target_pc, next_pc as u64, block_map)
    }

    fn handle_branch_ge_u_unified(
        &self,
        translator: &mut Translator,
        format: &parser::format::RRO,
        pc: usize,
        next_pc: usize,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        let parser::format::RRO { reg0, reg1, off0 } = *format;
        let reg0_val = translator.builder.use_var(translator.registers[&reg0]);
        let reg1_val = translator.builder.use_var(translator.registers[&reg1]);
        let condition =
            translator
                .builder
                .ins()
                .icmp(IntCC::UnsignedGreaterThanOrEqual, reg0_val, reg1_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(translator, condition, target_pc, next_pc as u64, block_map)
    }

    fn handle_branch_ge_s_unified(
        &self,
        translator: &mut Translator,
        format: &parser::format::RRO,
        pc: usize,
        next_pc: usize,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        let parser::format::RRO { reg0, reg1, off0 } = *format;
        let reg0_val = translator.builder.use_var(translator.registers[&reg0]);
        let reg1_val = translator.builder.use_var(translator.registers[&reg1]);
        let condition =
            translator
                .builder
                .ins()
                .icmp(IntCC::SignedGreaterThanOrEqual, reg0_val, reg1_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(translator, condition, target_pc, next_pc as u64, block_map)
    }

    fn handle_branch_lt_u_imm_unified(
        &self,
        translator: &mut Translator,
        format: &parser::format::RIO,
        pc: usize,
        next_pc: usize,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        let parser::format::RIO { reg0, imm0, off0 } = *format;
        let reg_val = translator.builder.use_var(translator.registers[&reg0]);
        let imm_val = translator.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = translator
            .builder
            .ins()
            .icmp(IntCC::UnsignedLessThan, reg_val, imm_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(translator, condition, target_pc, next_pc as u64, block_map)
    }

    fn handle_branch_lt_s_imm_unified(
        &self,
        translator: &mut Translator,
        format: &parser::format::RIO,
        pc: usize,
        next_pc: usize,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        let parser::format::RIO { reg0, imm0, off0 } = *format;
        let reg_val = translator.builder.use_var(translator.registers[&reg0]);
        let imm_val = translator.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = translator
            .builder
            .ins()
            .icmp(IntCC::SignedLessThan, reg_val, imm_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(translator, condition, target_pc, next_pc as u64, block_map)
    }

    fn handle_branch_ge_u_imm_unified(
        &self,
        translator: &mut Translator,
        format: &parser::format::RIO,
        pc: usize,
        next_pc: usize,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        let parser::format::RIO { reg0, imm0, off0 } = *format;
        let reg_val = translator.builder.use_var(translator.registers[&reg0]);
        let imm_val = translator.builder.ins().iconst(types::I64, imm0 as i64);
        let condition =
            translator
                .builder
                .ins()
                .icmp(IntCC::UnsignedGreaterThanOrEqual, reg_val, imm_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(translator, condition, target_pc, next_pc as u64, block_map)
    }

    fn handle_branch_ge_s_imm_unified(
        &self,
        translator: &mut Translator,
        format: &parser::format::RIO,
        pc: usize,
        next_pc: usize,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        let parser::format::RIO { reg0, imm0, off0 } = *format;
        let reg_val = translator.builder.use_var(translator.registers[&reg0]);
        let imm_val = translator.builder.ins().iconst(types::I64, imm0 as i64);
        let condition =
            translator
                .builder
                .ins()
                .icmp(IntCC::SignedGreaterThanOrEqual, reg_val, imm_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(translator, condition, target_pc, next_pc as u64, block_map)
    }

    fn handle_branch_gt_u_imm_unified(
        &self,
        translator: &mut Translator,
        format: &parser::format::RIO,
        pc: usize,
        next_pc: usize,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        let parser::format::RIO { reg0, imm0, off0 } = *format;
        let reg_val = translator.builder.use_var(translator.registers[&reg0]);
        let imm_val = translator.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = translator
            .builder
            .ins()
            .icmp(IntCC::UnsignedGreaterThan, reg_val, imm_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(translator, condition, target_pc, next_pc as u64, block_map)
    }

    fn handle_branch_gt_s_imm_unified(
        &self,
        translator: &mut Translator,
        format: &parser::format::RIO,
        pc: usize,
        next_pc: usize,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        let parser::format::RIO { reg0, imm0, off0 } = *format;
        let reg_val = translator.builder.use_var(translator.registers[&reg0]);
        let imm_val = translator.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = translator
            .builder
            .ins()
            .icmp(IntCC::SignedGreaterThan, reg_val, imm_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(translator, condition, target_pc, next_pc as u64, block_map)
    }

    fn handle_branch_le_u_imm_unified(
        &self,
        translator: &mut Translator,
        format: &parser::format::RIO,
        pc: usize,
        next_pc: usize,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        let parser::format::RIO { reg0, imm0, off0 } = *format;
        let reg_val = translator.builder.use_var(translator.registers[&reg0]);
        let imm_val = translator.builder.ins().iconst(types::I64, imm0 as i64);
        let condition =
            translator
                .builder
                .ins()
                .icmp(IntCC::UnsignedLessThanOrEqual, reg_val, imm_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(translator, condition, target_pc, next_pc as u64, block_map)
    }

    fn handle_branch_le_s_imm_unified(
        &self,
        translator: &mut Translator,
        format: &parser::format::RIO,
        pc: usize,
        next_pc: usize,
        block_map: &std::collections::HashMap<u64, cranelift::prelude::Block>,
    ) -> Result<()> {
        let parser::format::RIO { reg0, imm0, off0 } = *format;
        let reg_val = translator.builder.use_var(translator.registers[&reg0]);
        let imm_val = translator.builder.ins().iconst(types::I64, imm0 as i64);
        let condition =
            translator
                .builder
                .ins()
                .icmp(IntCC::SignedLessThanOrEqual, reg_val, imm_val);
        let target_pc = (pc as i64 + off0 as i64) as u64;
        self.generate_unified_branch(translator, condition, target_pc, next_pc as u64, block_map)
    }
}

/// Basic block - single entry/exit instruction sequence with pre-parsed instructions
pub struct Block {
    /// Start PC of the block
    pub start: usize,
    /// End PC of the block
    pub end: usize,
    /// Whether the block terminates
    pub terminates: bool,
    /// Pre-parsed instructions for this block
    pub instructions: Vec<parser::reader::Offset<parser::Instruction>>,
}

/// Compiled native code
#[derive(Debug, Clone)]
pub struct Code {
    /// Pointer to the compiled code
    pub ptr: *const u8,

    /// Size of the compiled code
    pub size: usize,
}
