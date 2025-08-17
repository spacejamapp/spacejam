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
    /// PVM registers (0 to MAX_REGISTER_INDEX)
    pub registers: HashMap<u8, Variable>,

    /// Cranelift function builder
    pub builder: &'a mut FunctionBuilder<'b>,

    // Block-based compilation state
    has_explicit_trap: bool,

    // Track current PC position during translation
    current_pc: usize,

    // Jump table for dynamic jumps (djump)
    jump_table: Vec<u64>,

    // Program data for instruction length calculations
    program: Vec<u8>,

    // Context pointer for boundary checking and runtime operations
    ctx_ptr: Option<Value>,

    // Unified compilation mode - suppresses terminator generation in visitor
    unified_mode: bool,
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

        Self {
            registers,
            builder,
            has_explicit_trap: false,
            current_pc: 0,
            jump_table,
            program: Vec::new(),
            ctx_ptr: None,
            unified_mode: false,
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
    /// This method accepts a pre-analyzed block from the JIT module to avoid duplicate parsing and analysis
    pub fn translate_block_with_instructions(
        &mut self,
        program: &[u8],
        block: &crate::jit::Block,
    ) -> Result<(), anyhow::Error> {
        // Store program data for instruction length calculations
        self.program = program.to_vec();

        if let Some(first_instr) = block.instructions.first() {
            self.current_pc = first_instr.range.start;
            tracing::debug!(
                "Translating block with {} pre-parsed instructions, starting at PC {}, terminates={}",
                block.instructions.len(),
                self.current_pc,
                block.terminates
            );
        }

        // Process each pre-parsed instruction in the block
        for (i, instruction_offset) in block.instructions.iter().enumerate() {
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

            // Use the visitor to compile the instruction
            self.visit(instruction_offset.value, instruction_offset.range.start)?;

            // For the last instruction in a terminating block, store its PC in the context
            let is_last_instruction = i == block.instructions.len() - 1;
            if block.terminates && is_last_instruction {
                tracing::trace!(
                    "Last instruction {} in terminating block at PC {}",
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
    /// Note: This method still needs parsing for individual instruction length calculations
    /// when called during branch instruction processing. This is unavoidable as instruction
    /// lengths are needed during compilation, not just during initial analysis.
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

    /// Helper function to get the linear memory base address from ExtendedContext
    pub fn get_memory_base(&mut self) -> Value {
        let context_ptr = self.ctx_ptr.expect("Context pointer not initialized");
        let memory_ptr_offset = self
            .builder
            .ins()
            .iconst(types::I64, context_offsets::MEMORY_PTR_OFFSET as i64);
        let memory_ptr_addr = self.builder.ins().iadd(context_ptr, memory_ptr_offset);
        self.builder
            .ins()
            .load(types::I64, MemFlags::new(), memory_ptr_addr, 0)
    }

    /// Generic helper function for all branch instructions - optimizes Cranelift IR generation
    /// Eliminates code duplication and ensures consistent branch handling patterns
    pub fn generate_branch_instruction(
        &mut self,
        condition: Value,
        pc: usize,
        off0: i64,
    ) -> Result<(), anyhow::Error> {
        // Calculate branch target PC using offset
        let target_pc = (pc as i64 + off0) as u64;
        // Calculate continue target PC (current PC + instruction length)
        let instr_len = self.get_instruction_length(pc)?;
        let continue_pc = (pc + instr_len) as u64;

        // Get context pointer to store result
        let context_ptr = self.ctx_ptr.expect("Context pointer not initialized");
        let result_offset = self
            .builder
            .ins()
            .iconst(types::I64, context_offsets::RESULT_OFFSET as i64);
        let result_addr = self.builder.ins().iadd(context_ptr, result_offset);

        // Store conditional result: Jump if condition is true, Continue if false
        let jump_discriminant = self
            .builder
            .ins()
            .iconst(types::I64, exec_result::JUMP as i64); // Jump variant
        let continue_discriminant = self
            .builder
            .ins()
            .iconst(types::I64, exec_result::CONTINUE as i64); // Continue variant
        let selected_discriminant =
            self.builder
                .ins()
                .select(condition, jump_discriminant, continue_discriminant);

        // Store the discriminant
        self.builder
            .ins()
            .store(MemFlags::new(), selected_discriminant, result_addr, 0);

        // Store conditional target PC: jump_target if taken, continue_target if not taken
        let target_pc_val = self.builder.ins().iconst(types::I64, target_pc as i64);
        let continue_pc_val = self.builder.ins().iconst(types::I64, continue_pc as i64);
        let selected_pc = self
            .builder
            .ins()
            .select(condition, target_pc_val, continue_pc_val);
        let data_offset = self.builder.ins().iconst(types::I64, 8);
        let data_addr = self.builder.ins().iadd(result_addr, data_offset);
        self.builder
            .ins()
            .store(MemFlags::new(), selected_pc, data_addr, 0);

        Ok(())
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

    /// Get the context pointer for unified compilation
    pub fn get_context_ptr(&self) -> Option<Value> {
        self.ctx_ptr
    }

    /// Enable unified compilation mode (suppresses terminator generation in visitor)
    pub fn set_unified_mode(&mut self, enabled: bool) {
        self.unified_mode = enabled;
    }

    /// Check if translator is in unified compilation mode
    pub fn is_unified_mode(&self) -> bool {
        self.unified_mode
    }

    /// Get context pointer for visitor operations - handles both unified and block-based modes
    pub fn get_context_ptr_for_visitor(&self) -> Value {
        if self.unified_mode {
            // In unified mode, use the stored context pointer
            self.ctx_ptr
                .expect("Context pointer not initialized in unified mode")
        } else {
            // In block-based mode, get from current block parameters
            self.builder
                .block_params(self.builder.current_block().unwrap())[0]
        }
    }
}
