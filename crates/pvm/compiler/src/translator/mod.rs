//! Translator module that converts PVM instructions to Cranelift IR

use crate::constants::{
    access, context_offsets, exec_result, BITS_PER_WORD, BITS_PER_WORD_SHIFT, BYTES_PER_U64_SHIFT,
    PAGE_SHIFT, PVM_REGISTER_COUNT,
};
use cranelift::prelude::*;
use parser::Visitor;
use std::collections::HashMap;

mod visitor;

/// PVM-to-Cranelift translator for block-based JIT compilation
pub struct Translator<'a, 'b> {
    pub registers: HashMap<u8, Variable>, // PVM registers (0 to MAX_REGISTER_INDEX)
    pub builder: &'a mut FunctionBuilder<'b>,

    // Block-based compilation state
    has_explicit_trap: bool, // Track if program contains explicit trap instructions
    current_pc: usize,       // Track current PC position during translation

    // Jump table for dynamic jumps (djump)
    jump_table: Vec<u64>,

    // Program data for instruction length calculations
    program: Vec<u8>,

    // Context pointer for boundary checking and runtime operations
    ctx_ptr: Option<Value>,
}

impl<'a, 'b> Translator<'a, 'b> {
    /// Create a new translator with PVM register variables and PC
    pub fn new(builder: &'a mut FunctionBuilder<'b>) -> Self {
        Self::with_jump_table(builder, Vec::new())
    }

    /// Create a new translator with PVM register variables, PC and jump table
    pub fn with_jump_table(builder: &'a mut FunctionBuilder<'b>, jump_table: Vec<u64>) -> Self {
        let mut registers = HashMap::new();

        // Declare all PVM registers as Cranelift variables
        // PVM has registers: ra(0), sp(1), unused(2,3,4), s0-s1(5-6), a0-a4(7-11), unused(12)
        for i in 0..PVM_REGISTER_COUNT {
            let var = Variable::new(i);
            builder.declare_var(var, types::I64);
            registers.insert(i as u8, var);
        }

        // PVM has only a fixed number of registers - no additional variables allowed!

        Self {
            registers,
            builder,
            has_explicit_trap: false,
            current_pc: 0,
            jump_table,
            program: Vec::new(),
            ctx_ptr: None,
        }
    }

    /// Initialize translator with runtime context (context loading handled by runtime)
    pub fn init_with_context(&mut self, context_ptr: Value) -> Result<(), anyhow::Error> {
        // Store context pointer for later use in boundary checking
        self.ctx_ptr = Some(context_ptr);
        // Block-based compilation: runtime handles context loading/saving
        // Registers are loaded from context before block execution
        // and saved back after block completion
        Ok(())
    }

    /// Translate a block of PVM instructions to Cranelift IR for block-based JIT
    /// This method is only used for block compilation, not whole-program compilation
    pub fn translate_block(
        &mut self,
        program: &[u8],
        start_pc: usize,
        end_pc: usize,
    ) -> Result<(), anyhow::Error> {
        // Store program data for instruction length calculations
        self.program = program.to_vec();

        tracing::debug!(
            "Translating block: start_pc={}, end_pc={}",
            start_pc,
            end_pc
        );

        let blob = parser::program::deblob(program)?;
        let mut reader = blob.reader();
        reader.set_position(start_pc);
        self.current_pc = start_pc;

        // Read the block of instructions using read_block method from parser
        // This reads until a terminating instruction or end of block
        let block_instructions = reader.read_block()?;

        // Process each instruction in the block
        for instruction_offset in block_instructions {
            // Skip instructions that are beyond our block boundary
            if instruction_offset.range.start >= end_pc {
                break;
            }

            // Log instruction compilation similar to interpreter
            tracing::trace!(
                "COMPILE PC={} opcode={}({:02x}) instruction={:?}",
                instruction_offset.range.start,
                if instruction_offset.range.start < program.len() {
                    program[instruction_offset.range.start]
                } else {
                    0
                },
                if instruction_offset.range.start < program.len() {
                    program[instruction_offset.range.start]
                } else {
                    0
                },
                instruction_offset.value
            );
            tracing::trace!(
                "COMPILE PC={:<6} {}",
                instruction_offset.range.start,
                instruction_offset.value
            );

            // Update current PC to track position - use the end of this instruction
            self.current_pc = instruction_offset.range.end;

            // Check if this is a terminating instruction and store its PC
            let is_terminating = matches!(
                instruction_offset.value,
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
            );

            // Use the visitor to compile the instruction
            self.visit(instruction_offset.value, instruction_offset.range.start)?;

            // For terminating instructions, store their PC in the context
            if is_terminating {
                tracing::trace!(
                    "Terminating instruction {} at PC {}",
                    instruction_offset.value,
                    instruction_offset.range.start
                );
                if let Some(ctx_ptr) = self.ctx_ptr {
                    self.store_instruction_pc(ctx_ptr, instruction_offset.range.start)?;
                }
            }
        }

        Ok(())
    }

    /// Calculate the length of a PVM instruction at the given PC
    /// This is crucial for correct PC advancement when branches are not taken
    pub fn get_instruction_length(&self, pc: usize) -> Result<usize, anyhow::Error> {
        if self.program.is_empty() {
            return Err(anyhow::anyhow!(
                "Program data not available for instruction length calculation"
            ));
        }

        let blob = parser::program::deblob(&self.program)?;
        let mut reader = blob.reader();
        reader.set_position(pc);

        if reader.eof() {
            return Err(anyhow::anyhow!("PC {} beyond program bounds", pc));
        }

        let start_pos = reader.position;
        // Use read_block and take the first instruction to calculate length
        let block_instructions = reader.read_block()?;
        if block_instructions.is_empty() {
            return Err(anyhow::anyhow!("No instruction found at PC {}", pc));
        }
        let instruction = &block_instructions[0];
        let end_pos = instruction.range.end;

        Ok(end_pos - start_pos)
    }

    /// Get the final PC after block translation
    pub fn get_final_pc(&self) -> usize {
        self.current_pc
    }

    pub fn get_current_pc(&self) -> usize {
        self.current_pc
    }

    /// Generate Cranelift IR to check page boundaries before store operations
    /// Uses stored context pointer and simple boundary logic matching interpreter
    pub fn check_store_boundaries(
        &mut self,
        address: Value,
        size_bytes: u32,
    ) -> Result<(), anyhow::Error> {
        let ctx_ptr = self.ctx_ptr.expect("Context pointer not initialized");

        // Page size constant is already defined and available

        // Always check page allocation for stores, regardless of boundary crossing
        // Calculate page numbers for first and last byte of the store operation
        let start_page = self.get_page_number(address);
        let size_minus_one = self
            .builder
            .ins()
            .iconst(types::I64, (size_bytes - 1) as i64);
        let last_byte_addr = self.builder.ins().iadd(address, size_minus_one);
        let end_page = self.get_page_number(last_byte_addr);

        tracing::debug!("Boundary check: will verify page allocation for store operation");

        // Create blocks for control flow
        let check_start_page_block = self.builder.create_block();
        let check_end_page_block = self.builder.create_block();
        let continue_block = self.builder.create_block();
        let trap_block = self.builder.create_block();

        // Always check the start page first
        self.builder.ins().jump(check_start_page_block, &[]);

        // Check start page allocation and writability
        self.builder.switch_to_block(check_start_page_block);

        // Check if start page is allocated and writable
        let start_page_valid = self.check_page_allocated_and_writable(ctx_ptr, start_page)?;
        self.builder
            .ins()
            .brif(start_page_valid, check_end_page_block, &[], trap_block, &[]);

        // Check end page allocation and writability
        self.builder.switch_to_block(check_end_page_block);
        let end_page_valid = self.check_page_allocated_and_writable(ctx_ptr, end_page)?;
        self.builder
            .ins()
            .brif(end_page_valid, continue_block, &[], trap_block, &[]);

        // Trap block: set page fault result and return
        self.builder.switch_to_block(trap_block);
        self.set_trap_result(ctx_ptr)?;
        self.builder.ins().return_(&[]);

        // Continue block: proceed with store operation
        self.builder.switch_to_block(continue_block);

        // Seal all created blocks
        self.builder.seal_block(check_start_page_block);
        self.builder.seal_block(check_end_page_block);
        self.builder.seal_block(continue_block);
        self.builder.seal_block(trap_block);

        Ok(())
    }

    /// Calculate page number from address using efficient bit shift
    fn get_page_number(&mut self, address: Value) -> Value {
        // Use bit shift for much faster page number calculation
        let shift_amount = self.builder.ins().iconst(types::I64, PAGE_SHIFT as i64);
        self.builder.ins().ushr(address, shift_amount)
    }

    /// Check if a page is allocated and writable by consulting the page bitmap and access array
    fn check_page_allocated_and_writable(
        &mut self,
        ctx_ptr: Value,
        page_num: Value,
    ) -> Result<Value, anyhow::Error> {
        // Use safer offset calculations based on ExtendedContext layout

        // Get page bitmap and access pointers from context
        let bitmap_offset = self
            .builder
            .ins()
            .iconst(types::I64, context_offsets::PAGE_BITMAP_OFFSET as i64);
        let bitmap_ptr_addr = self.builder.ins().iadd(ctx_ptr, bitmap_offset);
        let bitmap_ptr = self
            .builder
            .ins()
            .load(types::I64, MemFlags::new(), bitmap_ptr_addr, 0);

        let access_offset = self
            .builder
            .ins()
            .iconst(types::I64, context_offsets::PAGE_ACCESS_OFFSET as i64);
        let access_ptr_addr = self.builder.ins().iadd(ctx_ptr, access_offset);
        let access_ptr = self
            .builder
            .ins()
            .load(types::I64, MemFlags::new(), access_ptr_addr, 0);

        // Check if page is allocated (bit set in bitmap)
        // Use bit shifts for faster bitmap indexing
        let shift_bits = self
            .builder
            .ins()
            .iconst(types::I64, BITS_PER_WORD_SHIFT as i64);
        let word_idx = self.builder.ins().ushr(page_num, shift_bits); // page_num / BITS_PER_WORD
        let mask = self
            .builder
            .ins()
            .iconst(types::I64, (BITS_PER_WORD - 1) as i64);
        let bit_idx = self.builder.ins().band(page_num, mask); // page_num % BITS_PER_WORD

        // Load the bitmap word
        // Use bit shift for word offset: 8 bytes per u64
        let byte_shift = self
            .builder
            .ins()
            .iconst(types::I64, BYTES_PER_U64_SHIFT as i64);
        let word_offset = self.builder.ins().ishl(word_idx, byte_shift); // word_idx * 8
        let word_addr = self.builder.ins().iadd(bitmap_ptr, word_offset);
        let bitmap_word = self
            .builder
            .ins()
            .load(types::I64, MemFlags::new(), word_addr, 0);

        // Check if the bit is set (page allocated)
        let one_val = self.builder.ins().iconst(types::I64, 1);
        let bit_mask = self.builder.ins().ishl(one_val, bit_idx);
        let bit_value = self.builder.ins().band(bitmap_word, bit_mask);
        let zero_val = self.builder.ins().iconst(types::I64, 0);
        let page_allocated = self
            .builder
            .ins()
            .icmp(IntCC::NotEqual, bit_value, zero_val);

        // Check if page is writable (access == MUTABLE)
        let access_addr = self.builder.ins().iadd(access_ptr, page_num);
        let access_byte = self
            .builder
            .ins()
            .load(types::I8, MemFlags::new(), access_addr, 0);
        let mutable_byte = self.builder.ins().iconst(types::I8, access::MUTABLE as i64);
        let page_writable = self
            .builder
            .ins()
            .icmp(IntCC::Equal, access_byte, mutable_byte);

        // Page is valid if both allocated and writable
        let page_valid = self.builder.ins().band(page_allocated, page_writable);

        Ok(page_valid)
    }

    /// Set trap result in the context
    fn set_trap_result(&mut self, ctx_ptr: Value) -> Result<(), anyhow::Error> {
        let result_offset = self
            .builder
            .ins()
            .iconst(types::I64, context_offsets::RESULT_OFFSET as i64);
        let result_addr = self.builder.ins().iadd(ctx_ptr, result_offset);

        let trap_discriminant = self
            .builder
            .ins()
            .iconst(types::I64, exec_result::TRAP as i64);
        self.builder
            .ins()
            .store(MemFlags::new(), trap_discriminant, result_addr, 0);

        // For page faults specifically, set PC to block start
        // But for other traps, PC is already set correctly by the instruction visitor
        // TODO: Only set PC for page fault traps, not all traps

        Ok(())
    }

    /// Store the current instruction PC in the context for terminating instructions
    pub fn store_instruction_pc(
        &mut self,
        ctx_ptr: Value,
        instruction_pc: usize,
    ) -> Result<(), anyhow::Error> {
        let pc_offset = self
            .builder
            .ins()
            .iconst(types::I64, context_offsets::PC_OFFSET as i64);
        let pc_addr = self.builder.ins().iadd(ctx_ptr, pc_offset);
        let pc_val = self.builder.ins().iconst(types::I64, instruction_pc as i64);
        self.builder
            .ins()
            .store(MemFlags::new(), pc_val, pc_addr, 0);
        Ok(())
    }
}
