//! Translator module that converts PVM instructions to Cranelift IR

use cranelift::prelude::*;
use parser::Visitor;
use std::collections::HashMap;

mod visitor;

/// PVM-to-Cranelift translator for block-based JIT compilation
pub struct Translator<'a, 'b> {
    pub registers: HashMap<u8, Variable>, // Only 13 PVM registers (0-12)
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

        // Declare all 13 PVM registers as Cranelift variables
        // PVM has 13 registers: ra(0), sp(1), unused(2,3,4), s0-s1(5-6), a0-a4(7-11), unused(12)
        for i in 0..13 {
            let var = Variable::new(i);
            builder.declare_var(var, types::I64);
            registers.insert(i as u8, var);
        }

        // PVM has ONLY 13 registers - no additional variables allowed!

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

        // Process instructions in this block linearly
        while !reader.eof() && reader.position < end_pc {
            let instruction_offset = reader.read()?;
            if instruction_offset.range.start >= end_pc {
                break;
            }

            // Log instruction compilation similar to interpreter
            eprintln!(
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

            // Update current PC to track position
            self.current_pc = reader.position;

            // Use the visitor to compile the instruction
            self.visit(instruction_offset.value, instruction_offset.range.start)?;
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
        let _instruction = reader.read()?; // Read the instruction to calculate length
        let end_pos = reader.position;

        Ok(end_pos - start_pos)
    }

    /// Get the final PC after block translation
    pub fn get_final_pc(&self) -> usize {
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

        // Cache page size constant for better performance
        const PAGE_SIZE_CONST: i64 = 4096;
        let page_size = self.builder.ins().iconst(types::I64, PAGE_SIZE_CONST);
        
        // Check if store crosses page boundary (4KB pages)
        let page_offset = self.builder.ins().urem(address, page_size);
        let size_val = self.builder.ins().iconst(types::I64, size_bytes as i64);
        let end_offset = self.builder.ins().iadd(page_offset, size_val);

        // Check if end_offset > page_size (crosses boundary into next page)
        let crosses_boundary =
            self.builder
                .ins()
                .icmp(IntCC::UnsignedGreaterThan, end_offset, page_size);

        // Create blocks for control flow
        let check_block = self.builder.create_block();
        let continue_block = self.builder.create_block();

        // Branch: if crosses boundary, need to check; otherwise continue
        self.builder
            .ins()
            .brif(crosses_boundary, check_block, &[], continue_block, &[]);

        // Check block: verify the next page exists and is writable
        self.builder.switch_to_block(check_block);

        // Calculate page number of the last byte that will be written
        // Add overflow protection as recommended by expert
        let size_minus_one = self.builder.ins().iconst(types::I64, (size_bytes - 1) as i64);
        let max_address = self.builder.ins().iconst(types::I64, u32::MAX as i64);
        
        // Check for address overflow before addition
        let remaining_space = self.builder.ins().isub(max_address, address);
        let will_overflow = self.builder.ins().icmp(IntCC::UnsignedLessThan, remaining_space, size_minus_one);
        
        let overflow_trap_block = self.builder.create_block();
        let safe_calc_block = self.builder.create_block();
        
        // Branch: if overflow would occur, trap; otherwise continue with calculation
        self.builder
            .ins()
            .brif(will_overflow, overflow_trap_block, &[], safe_calc_block, &[]);

        // Overflow trap block
        self.builder.switch_to_block(overflow_trap_block);
        self.set_trap_result(ctx_ptr)?;
        self.builder.ins().return_(&[]);

        // Safe calculation block
        self.builder.switch_to_block(safe_calc_block);
        let last_byte_addr = self.builder.ins().iadd(address, size_minus_one);
        let last_byte_page = self.get_page_number(last_byte_addr);

        // Check if the last byte's page is allocated and writable
        let page_valid = self.check_page_allocated_and_writable(ctx_ptr, last_byte_page)?;

        let trap_block = self.builder.create_block();

        // Branch: if page is valid, continue; otherwise trap
        self.builder
            .ins()
            .brif(page_valid, continue_block, &[], trap_block, &[]);

        // Trap block: set page fault result and return
        self.builder.switch_to_block(trap_block);
        self.set_trap_result(ctx_ptr)?;
        self.builder.ins().return_(&[]);

        // Continue block: proceed with store operation
        self.builder.switch_to_block(continue_block);

        // Seal all created blocks
        self.builder.seal_block(check_block);
        self.builder.seal_block(safe_calc_block);
        self.builder.seal_block(overflow_trap_block);
        self.builder.seal_block(continue_block);
        self.builder.seal_block(trap_block);

        Ok(())
    }

    /// Calculate page number from address using efficient bit shift
    /// PAGE_SIZE = 4096 = 2^12, so division by PAGE_SIZE == right shift by 12 bits
    fn get_page_number(&mut self, address: Value) -> Value {
        // Use bit shift for much faster page number calculation
        // 4096 = 2^12, so divide by 4096 == right shift by 12
        const PAGE_SHIFT: i64 = 12;
        let shift_amount = self.builder.ins().iconst(types::I64, PAGE_SHIFT);
        self.builder.ins().ushr(address, shift_amount)
    }

    /// Check if a page is allocated and writable by consulting the page bitmap and access array
    fn check_page_allocated_and_writable(
        &mut self,
        ctx_ptr: Value,
        page_num: Value,
    ) -> Result<Value, anyhow::Error> {
        // Use safer offset calculations based on ExtendedContext layout
        // ExtendedContext: registers[13*8] + pc[8] + memory_ptr[8] + page_bitmap[8] + page_access[8] + result + pc_managed
        const BITMAP_OFFSET: i64 = (13 * 8 + 8 + 8) as i64; // After registers, pc, memory_ptr
        const ACCESS_OFFSET: i64 = (13 * 8 + 8 + 8 + 8) as i64; // After registers, pc, memory_ptr, page_bitmap
        
        // Get page bitmap and access pointers from context
        let bitmap_offset = self.builder.ins().iconst(types::I64, BITMAP_OFFSET);
        let bitmap_ptr_addr = self.builder.ins().iadd(ctx_ptr, bitmap_offset);
        let bitmap_ptr = self
            .builder
            .ins()
            .load(types::I64, MemFlags::new(), bitmap_ptr_addr, 0);

        let access_offset = self.builder.ins().iconst(types::I64, ACCESS_OFFSET);
        let access_ptr_addr = self.builder.ins().iadd(ctx_ptr, access_offset);
        let access_ptr = self
            .builder
            .ins()
            .load(types::I64, MemFlags::new(), access_ptr_addr, 0);

        // Check if page is allocated (bit set in bitmap)
        // Use bit shifts for faster bitmap indexing: 64 = 2^6
        let six = self.builder.ins().iconst(types::I64, 6); // 2^6 = 64
        let word_idx = self.builder.ins().ushr(page_num, six); // page_num / 64
        let sixty_three = self.builder.ins().iconst(types::I64, 63); // 64 - 1 = 63 (mask)
        let bit_idx = self.builder.ins().band(page_num, sixty_three); // page_num % 64

        // Load the bitmap word
        // Use bit shift for word offset: 8 bytes = 2^3, so * 8 == left shift by 3
        let three = self.builder.ins().iconst(types::I64, 3); // 2^3 = 8 bytes per u64
        let word_offset = self.builder.ins().ishl(word_idx, three); // word_idx * 8
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

        // Check if page is writable (access == 0)
        let access_addr = self.builder.ins().iadd(access_ptr, page_num);
        let access_byte = self
            .builder
            .ins()
            .load(types::I8, MemFlags::new(), access_addr, 0);
        let zero_byte = self.builder.ins().iconst(types::I8, 0);
        let page_writable = self
            .builder
            .ins()
            .icmp(IntCC::Equal, access_byte, zero_byte);

        // Page is valid if both allocated and writable
        let page_valid = self.builder.ins().band(page_allocated, page_writable);

        Ok(page_valid)
    }

    /// Set trap result in the context
    fn set_trap_result(&mut self, ctx_ptr: Value) -> Result<(), anyhow::Error> {
        let result_offset = self.builder.ins().iconst(
            types::I64,
            (std::mem::size_of::<[u64; 13]>()
                + std::mem::size_of::<u64>()
                + std::mem::size_of::<*mut u8>()
                + std::mem::size_of::<*const u64>()
                + std::mem::size_of::<*const u8>()) as i64,
        );
        let result_addr = self.builder.ins().iadd(ctx_ptr, result_offset);

        let trap_discriminant = self.builder.ins().iconst(types::I64, 3); // ExecResult::Trap
        self.builder
            .ins()
            .store(MemFlags::new(), trap_discriminant, result_addr, 0);

        Ok(())
    }
}
