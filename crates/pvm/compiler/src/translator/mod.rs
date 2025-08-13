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
    current_pc: usize, // Track current PC position during translation

    // Jump table for dynamic jumps (djump)
    jump_table: Vec<u64>,
    
    // Program data for instruction length calculations
    program: Vec<u8>,
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
        }
    }

    /// Initialize translator with runtime context (context loading handled by runtime)
    pub fn init_with_context(&mut self, _context_ptr: Value) -> Result<(), anyhow::Error> {
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
        
        tracing::debug!("Translating block: start_pc={}, end_pc={}", start_pc, end_pc);
        
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
            return Err(anyhow::anyhow!("Program data not available for instruction length calculation"));
        }
        
        let blob = parser::program::deblob(&self.program)?;
        let mut reader = blob.reader();
        reader.set_position(pc);
        
        if reader.eof() {
            return Err(anyhow::anyhow!("PC {} beyond program bounds", pc));
        }
        
        let start_pos = reader.position;
        let _instruction = reader.read()?;  // Read the instruction to calculate length
        let end_pos = reader.position;
        
        Ok(end_pos - start_pos)
    }

    /// Get the final PC after block translation
    pub fn get_final_pc(&self) -> usize {
        self.current_pc
    }
}
