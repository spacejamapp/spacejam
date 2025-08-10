//! Translator module that converts PVM instructions to Cranelift IR

use cranelift::prelude::*;
use parser::Visitor;
use std::collections::HashMap;

mod visitor;

/// Memory operation sizes for code generation
#[derive(Debug, Copy, Clone)]
pub enum MemorySize {
    Byte,  // 8 bits
    Word,  // 16 bits
    DWord, // 32 bits
    QWord, // 64 bits
}

/// PVM-to-Cranelift translator for block-based JIT compilation
pub struct Translator<'a, 'b> {
    pub registers: HashMap<u8, Variable>,
    pub pc: Variable,
    pub instruction_pc: Variable,  // PC of current instruction (for branch calculations)
    pub memory_ptr: Variable,
    pub context_var: Variable,    // Context variable for accessing runtime state
    pub builder: &'a mut FunctionBuilder<'b>,
    
    // Block-based compilation state
    has_explicit_trap: bool, // Track if program contains explicit trap instructions
}

impl<'a, 'b> Translator<'a, 'b> {
    /// Create a new translator with PVM register variables and PC
    pub fn new(builder: &'a mut FunctionBuilder<'b>) -> Self {
        let mut registers = HashMap::new();

        // Declare all 13 PVM registers as Cranelift variables
        // PVM has 13 registers: ra(0), sp(1), unused(2,3,4), s0-s1(5-6), a0-a4(7-11), unused(12)
        for i in 0..13 {
            let var = Variable::new(i);
            builder.declare_var(var, types::I64);
            registers.insert(i as u8, var);
        }

        // Declare PC variable (use variable index 13)
        let pc = Variable::new(13);
        builder.declare_var(pc, types::I64);

        // Declare instruction PC variable (use variable index 14)
        let instruction_pc = Variable::new(14);
        builder.declare_var(instruction_pc, types::I64);

        // Declare memory pointer variable (use variable index 15)
        let memory_ptr = Variable::new(15);
        builder.declare_var(memory_ptr, types::I64);

        // Declare context variable (use variable index 16)
        let context_var = Variable::new(16);
        builder.declare_var(context_var, types::I64);

        Self {
            registers,
            pc,
            instruction_pc,
            memory_ptr,
            context_var,
            builder,
            has_explicit_trap: false,
        }
    }

    /// Load initial execution context from ExtendedBlockContext
    pub fn load_initial_context(&mut self, context_ptr: Value) -> Result<(), anyhow::Error> {
        // Store context_ptr in a variable for use throughout translation
        self.builder.def_var(self.context_var, context_ptr);
        
        // ExtendedBlockContext layout:
        // - registers[13]: 0..104 (13 * 8 bytes)
        // - pc: 104..112 (8 bytes)
        // - memory_ptr: 112..120 (8 bytes)
        // - result: 120+ 
        
        // Load all 13 registers from context.registers
        for i in 0..13 {
            let offset = self.builder.ins().iconst(types::I64, (i * 8) as i64);
            let addr = self.builder.ins().iadd(context_ptr, offset);
            let value = self.builder.ins().load(types::I64, MemFlags::new(), addr, 0);
            let var = self.registers[&(i as u8)];
            self.builder.def_var(var, value);
        }

        // Load PC from context.pc (offset 104)
        let pc_offset = self.builder.ins().iconst(types::I64, 104);
        let pc_addr = self.builder.ins().iadd(context_ptr, pc_offset);
        let pc_value = self.builder.ins().load(types::I64, MemFlags::new(), pc_addr, 0);
        self.builder.def_var(self.pc, pc_value);
        
        // Initialize instruction_pc to the same value initially
        self.builder.def_var(self.instruction_pc, pc_value);

        // Load memory pointer from context.memory_ptr (offset 112)
        let mem_offset = self.builder.ins().iconst(types::I64, 112);
        let mem_addr = self.builder.ins().iadd(context_ptr, mem_offset);
        let mem_ptr_value = self.builder.ins().load(types::I64, MemFlags::new(), mem_addr, 0);
        self.builder.def_var(self.memory_ptr, mem_ptr_value);

        Ok(())
    }

    /// Set the block execution result to Jump with target PC
    pub fn set_jump_result(&mut self, target_pc: Value) -> Result<(), anyhow::Error> {
        let context_ptr = self.builder.use_var(self.context_var);
        
        // ExtendedBlockContext layout:
        // - registers[13]: 0..104
        // - pc: 104..112
        // - memory_ptr: 112..120
        // - result: 120..
        let result_offset = self.builder.ins().iconst(types::I64, 120);
        let result_addr = self.builder.ins().iadd(context_ptr, result_offset);
        
        // Set result discriminant to Jump (enum variant 1)
        let jump_discriminant = self.builder.ins().iconst(types::I64, 1);
        self.builder.ins().store(MemFlags::new(), jump_discriminant, result_addr, 0);
        
        // Set jump target PC at result + 8
        let target_offset = self.builder.ins().iconst(types::I64, 8);
        let target_addr = self.builder.ins().iadd(result_addr, target_offset);
        self.builder.ins().store(MemFlags::new(), target_pc, target_addr, 0);
        
        Ok(())
    }
    
    /// Set the block execution result to Trap
    pub fn set_trap_result(&mut self) -> Result<(), anyhow::Error> {
        let context_ptr = self.builder.use_var(self.context_var);
        
        // Set result discriminant to Trap (enum variant 3)
        let result_offset = self.builder.ins().iconst(types::I64, 120);
        let result_addr = self.builder.ins().iadd(context_ptr, result_offset);
        let trap_discriminant = self.builder.ins().iconst(types::I64, 3);
        self.builder.ins().store(MemFlags::new(), trap_discriminant, result_addr, 0);
        
        Ok(())
    }
    
    /// Set the block execution result to Halt
    pub fn set_halt_result(&mut self) -> Result<(), anyhow::Error> {
        let context_ptr = self.builder.use_var(self.context_var);
        
        // Set result discriminant to Halt (enum variant 2)
        let result_offset = self.builder.ins().iconst(types::I64, 120);
        let result_addr = self.builder.ins().iadd(context_ptr, result_offset);
        let halt_discriminant = self.builder.ins().iconst(types::I64, 2);
        self.builder.ins().store(MemFlags::new(), halt_discriminant, result_addr, 0);
        
        Ok(())
    }

    /// Translate a block of PVM instructions to Cranelift IR for block-based JIT
    /// This method is only used for block compilation, not whole-program compilation
    pub fn translate_block(&mut self, program: &[u8], start_pc: usize, end_pc: usize) -> Result<(), anyhow::Error> {
        let blob = parser::program::deblob(program)?;
        let mut reader = blob.reader();
        reader.set_position(start_pc);

        // Process instructions in this block linearly
        while !reader.eof() && reader.position < end_pc {
            let instruction_offset = reader.read()?;
            if instruction_offset.range.start >= end_pc {
                break;
            }

            // Store the instruction PC for branch calculations
            // Branch offsets are calculated from the instruction's PC, not the next PC
            let instruction_pc_value = self.builder.ins().iconst(types::I64, instruction_offset.range.start as i64);
            self.builder.def_var(self.instruction_pc, instruction_pc_value);
            
            // Increment PC by instruction size to point to next instruction  
            let instruction_size = self
                .builder
                .ins()
                .iconst(types::I64, instruction_offset.range.len() as i64);
            let new_pc = self.builder.ins().iadd(instruction_pc_value, instruction_size);
            self.builder.def_var(self.pc, new_pc);

            // Use the visitor to compile the instruction
            self.visit(instruction_offset.value)?;
        }

        Ok(())
    }

    /// Generate memory read with proper access validation
    pub fn emit_memory_read(&mut self, address: Value, size: MemorySize) -> Value {
        let memory_ptr = self.builder.use_var(self.memory_ptr);
        
        // Convert address to 32-bit and extend back to 64-bit for pointer arithmetic
        let addr_32 = self.builder.ins().ireduce(types::I32, address);
        let addr_64 = self.builder.ins().uextend(types::I64, addr_32);
        
        // Calculate actual memory address
        let read_addr = self.builder.ins().iadd(memory_ptr, addr_64);
        
        // Perform the actual read
        match size {
            MemorySize::Byte => {
                self.builder.ins().load(types::I8, MemFlags::new(), read_addr, 0)
            }
            MemorySize::Word => {
                self.builder.ins().load(types::I16, MemFlags::new(), read_addr, 0)
            }
            MemorySize::DWord => {
                self.builder.ins().load(types::I32, MemFlags::new(), read_addr, 0)
            }
            MemorySize::QWord => {
                self.builder.ins().load(types::I64, MemFlags::new(), read_addr, 0)
            }
        }
    }

    /// Generate signed memory read
    pub fn emit_memory_read_signed(&mut self, address: Value, size: MemorySize) -> Value {
        // Read the unsigned value first
        let unsigned_value = self.emit_memory_read(address, size);

        // Sign-extend to 64-bit based on the size
        match size {
            MemorySize::Byte => self.builder.ins().sextend(types::I64, unsigned_value),
            MemorySize::Word => self.builder.ins().sextend(types::I64, unsigned_value),
            MemorySize::DWord => self.builder.ins().sextend(types::I64, unsigned_value),
            MemorySize::QWord => unsigned_value, // Already 64-bit
        }
    }

    /// Generate memory write
    pub fn emit_memory_write(&mut self, address: Value, value: Value, size: MemorySize) {
        let memory_ptr = self.builder.use_var(self.memory_ptr);
        
        // Convert address to 32-bit and extend back to 64-bit for pointer arithmetic
        let addr_32 = self.builder.ins().ireduce(types::I32, address);
        let addr_64 = self.builder.ins().uextend(types::I64, addr_32);
        
        // Calculate actual memory address
        let write_addr = self.builder.ins().iadd(memory_ptr, addr_64);
        
        // Prepare value in correct size
        let write_value = match size {
            MemorySize::Byte => {
                if self.builder.func.dfg.value_type(value).bits() > 8 {
                    self.builder.ins().ireduce(types::I8, value)
                } else {
                    value
                }
            }
            MemorySize::Word => {
                if self.builder.func.dfg.value_type(value).bits() > 16 {
                    self.builder.ins().ireduce(types::I16, value)
                } else {
                    value
                }
            }
            MemorySize::DWord => {
                if self.builder.func.dfg.value_type(value).bits() > 32 {
                    self.builder.ins().ireduce(types::I32, value)
                } else {
                    value
                }
            }
            MemorySize::QWord => value,
        };
        
        // Perform the actual write
        self.builder.ins().store(MemFlags::new(), write_value, write_addr, 0);
    }
}