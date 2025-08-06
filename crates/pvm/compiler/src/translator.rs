//! Translator module that converts PVM instructions to Cranelift IR

use cranelift::prelude::*;
use parser::{format, Instruction, Visitor};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Memory operation sizes
#[derive(Debug, Copy, Clone)]
enum MemorySize {
    Byte,  // 8 bits
    Word,  // 16 bits
    DWord, // 32 bits
    QWord, // 64 bits
}

// Context structure offsets
const MEMORY_PTR_OFFSET: i64 = 112; // 13*8 + 8 = registers + pc
const MAX_MEMORY: i64 = 0x100000; // 1MB linear memory size
const OPS_COUNT_OFFSET: i64 = 1664; // memory_ops_count offset
const OPS_OFFSET: i64 = 128; // memory_ops array offset
const OP_SIZE: i64 = 24; // Size of each MemoryOp struct
const MAX_OPS: i32 = 64; // Maximum recorded memory operations

/// Temporary visitor wrapper to avoid lifetime issues
pub struct Translator<'a, 'b> {
    pub registers: HashMap<u8, Variable>,
    pub pc: Variable,
    pub memory_ptr: Variable,
    pub execution_mask: Variable, // Track which execution path we're on
    pub context_var: Variable,    // Context variable for accessing runtime state
    builder: &'a mut FunctionBuilder<'b>,

    // Control flow analysis
    basic_blocks: BTreeMap<usize, Block>, // PC offset -> Cranelift block
    branch_targets: BTreeSet<usize>,      // Set of all branch target offsets
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

        // Declare memory pointer variable (use variable index 14)
        let memory_ptr = Variable::new(14);
        builder.declare_var(memory_ptr, types::I64);

        // Declare execution mask variable (use variable index 15)
        let execution_mask = Variable::new(15);
        builder.declare_var(execution_mask, types::I8);

        // Declare context variable (use variable index 16)
        let context_var = Variable::new(16);
        builder.declare_var(context_var, types::I64);

        // Memory operations will be handled inline, not via external functions

        Self {
            registers,
            pc,
            memory_ptr,
            execution_mask,
            context_var,
            builder,
            basic_blocks: BTreeMap::new(),
            branch_targets: BTreeSet::new(),
        }
    }

    /// Load initial execution context (registers + PC) from memory pointer
    pub fn load_initial_context(&mut self, context_ptr: Value) -> Result<(), anyhow::Error> {
        // Store context_ptr in a variable for use throughout translation
        self.builder.def_var(self.context_var, context_ptr);
        // Load all 13 registers from context.registers
        for i in 0..13 {
            let offset = self.builder.ins().iconst(types::I64, (i * 8) as i64);
            let addr = self.builder.ins().iadd(context_ptr, offset);
            let value = self
                .builder
                .ins()
                .load(types::I64, MemFlags::new(), addr, 0);
            let var = self.registers[&(i as u8)];
            self.builder.def_var(var, value);
        }

        // Load PC from context.pc (offset 13 * 8 = 104 bytes after start)
        let pc_offset = self.builder.ins().iconst(types::I64, 104);
        let pc_addr = self.builder.ins().iadd(context_ptr, pc_offset);
        let pc_value = self
            .builder
            .ins()
            .load(types::I64, MemFlags::new(), pc_addr, 0);
        self.builder.def_var(self.pc, pc_value);

        // Load memory pointer from context.memory_ptr (offset 112 bytes after start)
        let mem_offset = self.builder.ins().iconst(types::I64, 112);
        let mem_addr = self.builder.ins().iadd(context_ptr, mem_offset);
        let mem_ptr_value = self
            .builder
            .ins()
            .load(types::I64, MemFlags::new(), mem_addr, 0);
        self.builder.def_var(self.memory_ptr, mem_ptr_value);

        // Initialize execution mask to true (all instructions execute initially)
        let true_val = self.builder.ins().iconst(types::I8, 1);
        self.builder.def_var(self.execution_mask, true_val);

        Ok(())
    }

    /// Translate a PVM program to Cranelift IR and return final context values
    pub fn translate(&mut self, program: &[u8]) -> Result<(Vec<Value>, Value), anyhow::Error> {
        let blob = parser::program::deblob(program)?;
        let mut reader = blob.reader();

        // For simple programs without branches, use linear execution
        if self.has_control_flow(&blob)? {
            // Pass 1: Analyze control flow to identify basic block boundaries
            self.analyze_control_flow(&blob)?;

            // Pass 2: Generate code with proper control flow
            self.generate_code(&blob)?;
        } else {
            // Linear execution - no branches
            while !reader.eof() {
                let instruction_offset = reader.read()?;
                let instruction = instruction_offset.value;

                // Increment PC by instruction size before executing instruction
                let current_pc = self.builder.use_var(self.pc);
                let instruction_size = self
                    .builder
                    .ins()
                    .iconst(types::I64, instruction_offset.range.len() as i64);
                let new_pc = self.builder.ins().iadd(current_pc, instruction_size);
                self.builder.def_var(self.pc, new_pc);

                self.visit(instruction)?;
            }

            // Don't add return here - let the JIT handle it
        }

        // Return all 13 register values + PC (the JIT will handle the return instruction)
        let mut register_values = Vec::with_capacity(13);
        for i in 0..13 {
            let var = self.registers[&(i as u8)];
            register_values.push(self.builder.use_var(var));
        }

        let pc_value = self.builder.use_var(self.pc);

        Ok((register_values, pc_value))
    }

    /// Check if the program has any control flow instructions
    fn has_control_flow(&self, blob: &parser::program::ProgramBlob) -> Result<bool, anyhow::Error> {
        let mut reader = blob.reader();

        while !reader.eof() {
            let instruction_offset = reader.read()?;
            let instruction = &instruction_offset.value;

            match instruction {
                Instruction::Jump(_)
                | Instruction::JumpInd(_)
                | Instruction::LoadImmJump(_)
                | Instruction::LoadImmJumpInd(_)
                | Instruction::BranchEq(_)
                | Instruction::BranchNe(_)
                | Instruction::BranchLtU(_)
                | Instruction::BranchLtS(_)
                | Instruction::BranchGeU(_)
                | Instruction::BranchGeS(_)
                | Instruction::BranchEqImm(_)
                | Instruction::BranchNeImm(_)
                | Instruction::BranchLtUImm(_)
                | Instruction::BranchLtSImm(_)
                | Instruction::BranchGeUImm(_)
                | Instruction::BranchGeSImm(_)
                | Instruction::BranchLeUImm(_)
                | Instruction::BranchLeSImm(_)
                | Instruction::BranchGtUImm(_)
                | Instruction::BranchGtSImm(_) => {
                    return Ok(true);
                }
                _ => {}
            }
        }

        Ok(false)
    }

    /// Pass 1: Analyze the program to identify all branch targets and basic block boundaries
    fn analyze_control_flow(
        &mut self,
        blob: &parser::program::ProgramBlob,
    ) -> Result<(), anyhow::Error> {
        let mut reader = blob.reader();

        // Always start at offset 0
        self.branch_targets.insert(0);

        while !reader.eof() {
            let instruction_offset = reader.read()?;
            let current_pc = instruction_offset.range.start;
            let instruction = &instruction_offset.value;

            // Check if this instruction is a control flow instruction
            match instruction {
                Instruction::Jump(format) => {
                    let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                    self.branch_targets.insert(target_offset);
                    // Next instruction after jump is also a basic block start
                    self.branch_targets.insert(instruction_offset.range.end);
                }
                Instruction::BranchEq(format)
                | Instruction::BranchNe(format)
                | Instruction::BranchLtU(format)
                | Instruction::BranchLtS(format)
                | Instruction::BranchGeU(format)
                | Instruction::BranchGeS(format) => {
                    let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                    self.branch_targets.insert(target_offset);
                    // Next instruction after branch is also a basic block start (fall-through path)
                    self.branch_targets.insert(instruction_offset.range.end);
                }
                Instruction::BranchEqImm(format)
                | Instruction::BranchNeImm(format)
                | Instruction::BranchLtUImm(format)
                | Instruction::BranchLtSImm(format)
                | Instruction::BranchGeUImm(format)
                | Instruction::BranchGeSImm(format)
                | Instruction::BranchLeUImm(format)
                | Instruction::BranchLeSImm(format)
                | Instruction::BranchGtUImm(format)
                | Instruction::BranchGtSImm(format) => {
                    let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                    self.branch_targets.insert(target_offset);
                    // Next instruction after branch is also a basic block start (fall-through path)
                    self.branch_targets.insert(instruction_offset.range.end);
                }
                _ => {} // Not a control flow instruction
            }
        }

        Ok(())
    }

    /// Pass 2: Generate Cranelift IR with proper basic blocks
    fn generate_code(&mut self, blob: &parser::program::ProgramBlob) -> Result<(), anyhow::Error> {
        // Create Cranelift blocks for all branch targets
        for &target_offset in &self.branch_targets {
            let block = self.builder.create_block();
            self.basic_blocks.insert(target_offset, block);
        }

        // Create exit block for function termination
        let exit_block = self.builder.create_block();

        // Start with the entry block (offset 0)
        let entry_block = self.basic_blocks[&0];
        self.builder.switch_to_block(entry_block);

        let mut reader = blob.reader();
        let mut current_block_start = 0;
        let mut needs_fallthrough = true;

        while !reader.eof() {
            let instruction_offset = reader.read()?;
            let current_pc = instruction_offset.range.start;
            let instruction = instruction_offset.value;

            // Check if we need to switch to a new basic block
            if current_pc != current_block_start && self.branch_targets.contains(&current_pc) {
                // Add fallthrough jump if the previous block didn't terminate
                if needs_fallthrough {
                    self.builder.ins().jump(exit_block, &[]);
                }

                // We've reached a new basic block target - switch to it
                let target_block = self.basic_blocks[&current_pc];
                self.builder.switch_to_block(target_block);
                current_block_start = current_pc;
            }

            // Increment PC by instruction size before executing instruction
            let current_pc_val = self.builder.use_var(self.pc);
            let instruction_size = self
                .builder
                .ins()
                .iconst(types::I64, instruction_offset.range.len() as i64);
            let new_pc = self.builder.ins().iadd(current_pc_val, instruction_size);
            self.builder.def_var(self.pc, new_pc);

            // Execute the instruction
            needs_fallthrough = self.visit_with_control_flow(
                instruction,
                current_pc,
                instruction_offset.range.end,
            )?;
        }

        // If the last block needs fallthrough, jump to exit
        if needs_fallthrough {
            self.builder.ins().jump(exit_block, &[]);
        }

        // Switch to exit block - the JIT will handle the return
        self.builder.switch_to_block(exit_block);

        Ok(())
    }

    /// Execute an instruction with proper control flow handling
    /// Returns true if the block still needs fallthrough, false if it's terminated
    fn visit_with_control_flow(
        &mut self,
        instruction: Instruction,
        current_pc: usize,
        next_pc: usize,
    ) -> Result<bool, anyhow::Error> {
        match instruction {
            // Control flow instructions need special handling
            Instruction::Jump(format) => {
                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];
                self.builder.ins().jump(target_block, &[]);
                Ok(false) // Block is terminated, no fallthrough needed
            }
            Instruction::BranchEq(format) => {
                let reg0_var = self.registers[&format.reg0];
                let reg1_var = self.registers[&format.reg1];
                let reg0_val = self.builder.use_var(reg0_var);
                let reg1_val = self.builder.use_var(reg1_var);
                let condition = self.builder.ins().icmp(IntCC::Equal, reg0_val, reg1_val);

                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];
                let fallthrough_block = self.basic_blocks[&next_pc];

                self.builder
                    .ins()
                    .brif(condition, target_block, &[], fallthrough_block, &[]);
                Ok(false) // Block is terminated with conditional branch
            }
            Instruction::BranchNe(format) => {
                let reg0_var = self.registers[&format.reg0];
                let reg1_var = self.registers[&format.reg1];
                let reg0_val = self.builder.use_var(reg0_var);
                let reg1_val = self.builder.use_var(reg1_var);
                let condition = self.builder.ins().icmp(IntCC::NotEqual, reg0_val, reg1_val);

                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];
                let fallthrough_block = self.basic_blocks[&next_pc];

                self.builder
                    .ins()
                    .brif(condition, target_block, &[], fallthrough_block, &[]);
                Ok(false) // Block is terminated with conditional branch
            }
            Instruction::BranchEqImm(format) => {
                let reg0_var = self.registers[&format.reg0];
                let reg0_val = self.builder.use_var(reg0_var);
                let imm_val = self.builder.ins().iconst(types::I64, format.imm0 as i64);
                let condition = self.builder.ins().icmp(IntCC::Equal, reg0_val, imm_val);

                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];
                let fallthrough_block = self.basic_blocks[&next_pc];

                self.builder
                    .ins()
                    .brif(condition, target_block, &[], fallthrough_block, &[]);
                Ok(false) // Block is terminated with conditional branch
            }
            Instruction::BranchNeImm(format) => {
                let reg0_var = self.registers[&format.reg0];
                let reg0_val = self.builder.use_var(reg0_var);
                let imm_val = self.builder.ins().iconst(types::I64, format.imm0 as i64);
                let condition = self.builder.ins().icmp(IntCC::NotEqual, reg0_val, imm_val);

                let target_offset = (current_pc as i64 + format.off0 as i64) as usize;
                let target_block = self.basic_blocks[&target_offset];
                let fallthrough_block = self.basic_blocks[&next_pc];

                self.builder
                    .ins()
                    .brif(condition, target_block, &[], fallthrough_block, &[]);
                Ok(false) // Block is terminated with conditional branch
            }
            // Add more branch instructions as needed...
            _ => {
                // For non-control-flow instructions, use the existing visitor
                self.visit(instruction)?;
                Ok(true) // Block still needs fallthrough
            }
        }
    }
}

impl Translator<'_, '_> {
    /// Generate direct linear memory read
    fn emit_memory_read(&mut self, address: Value, size: MemorySize) -> Value {
        // Get the context pointer - use current block if it has params, otherwise entry block
        let context_param = if let Some(current_block) = self.builder.current_block() {
            let params = self.builder.block_params(current_block);
            if !params.is_empty() {
                params[0]
            } else {
                // Fall back to entry block
                let entry_block = self.builder.func.layout.blocks().nth(0).unwrap();
                self.builder.block_params(entry_block)[0]
            }
        } else {
            // Fall back to entry block
            let entry_block = self.builder.func.layout.blocks().nth(0).unwrap();
            self.builder.block_params(entry_block)[0]
        };

        // Convert address to 32-bit for PVM memory addressing
        let addr_32 = self.builder.ins().ireduce(types::I32, address);
        let addr_64 = self.builder.ins().uextend(types::I64, addr_32);

        // Load linear memory pointer from context
        let mem_ptr_addr = self
            .builder
            .ins()
            .iadd_imm(context_param, MEMORY_PTR_OFFSET);
        let memory_ptr = self
            .builder
            .ins()
            .load(types::I64, MemFlags::new(), mem_ptr_addr, 0);

        // Bounds check: address < MAX_MEMORY
        let in_bounds = self
            .builder
            .ins()
            .icmp_imm(IntCC::UnsignedLessThan, addr_64, MAX_MEMORY);

        // Calculate read address
        let read_addr = self.builder.ins().iadd(memory_ptr, addr_64);

        // Load from linear memory with proper type
        let loaded_value = match size {
            MemorySize::Byte => self
                .builder
                .ins()
                .load(types::I8, MemFlags::new(), read_addr, 0),
            MemorySize::Word => self
                .builder
                .ins()
                .load(types::I16, MemFlags::new(), read_addr, 0),
            MemorySize::DWord => self
                .builder
                .ins()
                .load(types::I32, MemFlags::new(), read_addr, 0),
            MemorySize::QWord => self
                .builder
                .ins()
                .load(types::I64, MemFlags::new(), read_addr, 0),
        };

        // Return 0 if out of bounds, otherwise return the loaded value
        let zero_value = match size {
            MemorySize::Byte => self.builder.ins().iconst(types::I8, 0),
            MemorySize::Word => self.builder.ins().iconst(types::I16, 0),
            MemorySize::DWord => self.builder.ins().iconst(types::I32, 0),
            MemorySize::QWord => self.builder.ins().iconst(types::I64, 0),
        };

        self.builder
            .ins()
            .select(in_bounds, loaded_value, zero_value)
    }

    fn emit_memory_read_signed(&mut self, address: Value, size: MemorySize) -> Value {
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

    /// Generate direct memory write using inline pointer dereferencing
    fn emit_memory_write(&mut self, address: Value, value: Value, size: MemorySize) {
        // Get the context pointer - use current block if it has params, otherwise entry block
        let context_param = if let Some(current_block) = self.builder.current_block() {
            let params = self.builder.block_params(current_block);
            if !params.is_empty() {
                params[0]
            } else {
                // Fall back to entry block
                let entry_block = self.builder.func.layout.blocks().nth(0).unwrap();
                self.builder.block_params(entry_block)[0]
            }
        } else {
            // Fall back to entry block
            let entry_block = self.builder.func.layout.blocks().nth(0).unwrap();
            self.builder.block_params(entry_block)[0]
        };

        // Convert address to 32-bit for PVM memory addressing
        let addr_32 = self.builder.ins().ireduce(types::I32, address);

        // Prepare value in correct type
        let value_type = self.builder.func.dfg.value_type(value);
        let write_value = match size {
            MemorySize::Byte => {
                if value_type.bits() > 8 {
                    self.builder.ins().ireduce(types::I8, value)
                } else {
                    value
                }
            }
            MemorySize::Word => {
                if value_type.bits() > 16 {
                    self.builder.ins().ireduce(types::I16, value)
                } else {
                    value
                }
            }
            MemorySize::DWord => {
                if value_type.bits() > 32 {
                    self.builder.ins().ireduce(types::I32, value)
                } else {
                    value
                }
            }
            MemorySize::QWord => value,
        };

        // Record the memory write operation in the context for post-execution processing
        // This is a simplified approach that avoids complex JIT memory integration

        // Convert value to 64-bit for storage
        let value_64 = match size {
            MemorySize::Byte | MemorySize::Word | MemorySize::DWord => {
                self.builder.ins().uextend(types::I64, write_value)
            }
            MemorySize::QWord => write_value,
        };

        // Size byte
        let size_byte = match size {
            MemorySize::Byte => 1u8,
            MemorySize::Word => 2u8,
            MemorySize::DWord => 4u8,
            MemorySize::QWord => 8u8,
        };
        let size_val = self.builder.ins().iconst(types::I8, size_byte as i64);
        let is_write_val = self.builder.ins().iconst(types::I8, 1); // True for write

        // Use predefined context offsets
        let ops_count_offset = OPS_COUNT_OFFSET;
        let ops_offset = OPS_OFFSET;
        let op_size = OP_SIZE;

        // Load current memory_ops_count
        let ops_count_addr = self.builder.ins().iadd_imm(context_param, ops_count_offset);
        let current_count = self
            .builder
            .ins()
            .load(types::I32, MemFlags::new(), ops_count_addr, 0);

        // Check if we have space for recording (count < MAX_OPS)
        let max_ops = self.builder.ins().iconst(types::I32, MAX_OPS as i64);
        let has_space = self
            .builder
            .ins()
            .icmp(IntCC::UnsignedLessThan, current_count, max_ops);

        // Record operation for post-execution processing (trap handling in apply_memory_operations)
        let can_write = has_space;

        // Create blocks for conditional recording
        let record_block = self.builder.create_block();
        let done_block = self.builder.create_block();

        self.builder
            .ins()
            .brif(can_write, record_block, &[], done_block, &[]);

        // Record memory operation when we have space
        self.builder.switch_to_block(record_block);

        // Calculate address of current memory operation
        let count_64 = self.builder.ins().uextend(types::I64, current_count);
        let op_size_val = self.builder.ins().iconst(types::I64, op_size);
        let offset_from_ops = self.builder.ins().imul(count_64, op_size_val);
        let ops_base_addr = self.builder.ins().iadd_imm(context_param, ops_offset);
        let current_op_addr = self.builder.ins().iadd(ops_base_addr, offset_from_ops);

        // Store the memory operation fields
        self.builder
            .ins()
            .store(MemFlags::new(), addr_32, current_op_addr, 0); // address
        self.builder
            .ins()
            .store(MemFlags::new(), value_64, current_op_addr, 8); // value
        self.builder
            .ins()
            .store(MemFlags::new(), size_val, current_op_addr, 16); // size
        self.builder
            .ins()
            .store(MemFlags::new(), is_write_val, current_op_addr, 17); // is_write

        // Increment memory_ops_count
        let new_count = self.builder.ins().iadd_imm(current_count, 1);
        self.builder
            .ins()
            .store(MemFlags::new(), new_count, ops_count_addr, 0);

        self.builder.ins().jump(done_block, &[]);

        // Continue execution
        self.builder.switch_to_block(done_block);
        self.builder.seal_block(record_block);
        self.builder.seal_block(done_block);
    }
}

impl Visitor for Translator<'_, '_> {
    type Error = anyhow::Error;

    fn visit_trap(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_fallthrough(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_load_imm(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, imm_val);
        Ok(())
    }

    fn visit_load_imm_64(&mut self, format: format::REI) -> Result<(), Self::Error> {
        let format::REI { reg0, eimm0 } = format;
        let imm_val = self.builder.ins().iconst(types::I64, eimm0 as i64);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, imm_val);
        Ok(())
    }

    fn visit_load_imm_jump(&mut self, format: format::RIO) -> Result<(), Self::Error> {
        let format::RIO { reg0, off0, imm0 } = format;
        // Load immediate value
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, imm_val);

        // Jump by adding offset to current PC
        let current_pc = self.builder.use_var(self.pc);
        let offset_val = self.builder.ins().iconst(types::I64, off0 as i64);
        let target_pc = self.builder.ins().iadd(current_pc, offset_val);
        self.builder.def_var(self.pc, target_pc);
        Ok(())
    }

    fn visit_add_imm_32(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        if reg0 >= 13 || reg1 >= 13 {
            anyhow::bail!("Invalid register numbers: dst={}, src={}", reg0, reg1);
        }

        // Load source register, truncate to 32-bit, add immediate, sign extend to 64-bit
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        let imm_val = self.builder.ins().iconst(types::I32, imm0 as i64);
        let result_32 = self.builder.ins().iadd(src_32, imm_val);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_add_imm_64(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        if reg0 >= 13 || reg1 >= 13 {
            anyhow::bail!("Invalid register numbers: dst={}, src={}", reg0, reg1);
        }

        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().iadd(src_val, imm_val);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_add_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let src0_32 = self.builder.ins().ireduce(types::I32, src0_val);
        let src1_32 = self.builder.ins().ireduce(types::I32, src1_val);
        let result_32 = self.builder.ins().iadd(src0_32, src1_32);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_add_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let result = self.builder.ins().iadd(src0_val, src1_val);
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_sub_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let src0_32 = self.builder.ins().ireduce(types::I32, src0_val);
        let src1_32 = self.builder.ins().ireduce(types::I32, src1_val);
        let result_32 = self.builder.ins().isub(src0_32, src1_32);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_sub_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let result = self.builder.ins().isub(src0_val, src1_val);
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_mul_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let src0_32 = self.builder.ins().ireduce(types::I32, src0_val);
        let src1_32 = self.builder.ins().ireduce(types::I32, src1_val);
        let result_32 = self.builder.ins().imul(src0_32, src1_32);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_mul_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let result = self.builder.ins().imul(src0_val, src1_val);
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_mul_imm_32(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        let imm_val = self.builder.ins().iconst(types::I32, imm0 as i64);
        let result_32 = self.builder.ins().imul(src_32, imm_val);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_mul_imm_64(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().imul(src_val, imm_val);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_div_u_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend_var = self.registers[&reg0];
        let divisor_var = self.registers[&reg1];
        let dividend_val = self.builder.use_var(dividend_var);
        let divisor_val = self.builder.use_var(divisor_var);
        let dividend_32 = self.builder.ins().ireduce(types::I32, dividend_val);
        let divisor_32 = self.builder.ins().ireduce(types::I32, divisor_val);

        // Check for division by zero and return u64::MAX if so
        let zero = self.builder.ins().iconst(types::I32, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor_32, zero);
        let max_val = self.builder.ins().iconst(types::I64, u64::MAX as i64);

        // Use conditional blocks to avoid division by zero
        let one_32 = self.builder.ins().iconst(types::I32, 1);
        let safe_divisor = self.builder.ins().select(is_zero, one_32, divisor_32);
        let result_32 = self.builder.ins().udiv(dividend_32, safe_divisor);
        let result_32_ext = self.builder.ins().uextend(types::I64, result_32);
        let result = self.builder.ins().select(is_zero, max_val, result_32_ext);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_div_u_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        if reg0 >= 13 || reg1 >= 13 || reg2 >= 13 {
            anyhow::bail!("Invalid register numbers: {}, {}, {}, ", reg0, reg1, reg2);
        }

        let dividend_var = self.registers[&reg0];
        let divisor_var = self.registers[&reg1];
        let dividend_val = self.builder.use_var(dividend_var);
        let divisor_val = self.builder.use_var(divisor_var);

        // Check for division by zero and return u64::MAX if so
        let zero = self.builder.ins().iconst(types::I64, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor_val, zero);
        let max_val = self.builder.ins().iconst(types::I64, u64::MAX as i64);

        // Use conditional blocks to avoid division by zero
        let one_64 = self.builder.ins().iconst(types::I64, 1);
        let safe_divisor = self.builder.ins().select(is_zero, one_64, divisor_val);
        let result_div = self.builder.ins().udiv(dividend_val, safe_divisor);
        let result = self.builder.ins().select(is_zero, max_val, result_div);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_div_s_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend_var = self.registers[&reg0];
        let divisor_var = self.registers[&reg1];
        let dividend_val = self.builder.use_var(dividend_var);
        let divisor_val = self.builder.use_var(divisor_var);
        let dividend_32 = self.builder.ins().ireduce(types::I32, dividend_val);
        let divisor_32 = self.builder.ins().ireduce(types::I32, divisor_val);

        // Check for division by zero
        let zero = self.builder.ins().iconst(types::I32, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor_32, zero);

        // Check for overflow (i32::MIN / -1)
        let min_val_32 = self.builder.ins().iconst(types::I32, i32::MIN as i64);
        let neg_one = self.builder.ins().iconst(types::I32, -1);
        let is_min = self
            .builder
            .ins()
            .icmp(IntCC::Equal, dividend_32, min_val_32);
        let is_neg_one = self.builder.ins().icmp(IntCC::Equal, divisor_32, neg_one);
        let is_overflow = self.builder.ins().band(is_min, is_neg_one);

        let max_val = self.builder.ins().iconst(types::I64, u64::MAX as i64);
        let min_result = self.builder.ins().iconst(types::I64, i32::MIN as i64);

        // Use safe divisor to avoid division faults
        let one_32 = self.builder.ins().iconst(types::I32, 1);
        let safe_divisor = self.builder.ins().select(is_zero, one_32, divisor_32);
        let safe_divisor = self.builder.ins().select(is_overflow, one_32, safe_divisor);
        let result_32 = self.builder.ins().sdiv(dividend_32, safe_divisor);
        let result_32_ext = self.builder.ins().sextend(types::I64, result_32);

        // Return u64::MAX for div by zero, i32::MIN for overflow, otherwise result
        let result_or_overflow = self
            .builder
            .ins()
            .select(is_overflow, min_result, result_32_ext);
        let result = self
            .builder
            .ins()
            .select(is_zero, max_val, result_or_overflow);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_div_s_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend_var = self.registers[&reg0];
        let divisor_var = self.registers[&reg1];
        let dividend_val = self.builder.use_var(dividend_var);
        let divisor_val = self.builder.use_var(divisor_var);

        // Check for division by zero
        let zero = self.builder.ins().iconst(types::I64, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor_val, zero);

        // Check for overflow (i64::MIN / -1)
        let min_val_64 = self.builder.ins().iconst(types::I64, i64::MIN);
        let neg_one = self.builder.ins().iconst(types::I64, -1);
        let is_min = self
            .builder
            .ins()
            .icmp(IntCC::Equal, dividend_val, min_val_64);
        let is_neg_one = self.builder.ins().icmp(IntCC::Equal, divisor_val, neg_one);
        let is_overflow = self.builder.ins().band(is_min, is_neg_one);

        let max_val = self.builder.ins().iconst(types::I64, u64::MAX as i64);

        // Use safe divisor to avoid division faults
        let one_64 = self.builder.ins().iconst(types::I64, 1);
        let safe_divisor = self.builder.ins().select(is_zero, one_64, divisor_val);
        let safe_divisor = self.builder.ins().select(is_overflow, one_64, safe_divisor);
        let result_div = self.builder.ins().sdiv(dividend_val, safe_divisor);

        // Return u64::MAX for div by zero, original dividend for overflow, otherwise result
        let result_or_overflow = self
            .builder
            .ins()
            .select(is_overflow, dividend_val, result_div);
        let result = self
            .builder
            .ins()
            .select(is_zero, max_val, result_or_overflow);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_move_reg(&mut self, format: format::RR) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, src_val);
        Ok(())
    }

    fn visit_and(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let result = self.builder.ins().band(src0_val, src1_val);
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_and_imm(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().band(src_val, imm_val);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_or(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let result = self.builder.ins().bor(src0_val, src1_val);
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_or_imm(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().bor(src_val, imm_val);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_xor(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let result = self.builder.ins().bxor(src0_val, src1_val);
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_xor_imm(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().bxor(src_val, imm_val);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_rem_u_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend_var = self.registers[&reg0];
        let divisor_var = self.registers[&reg1];
        let dividend_val = self.builder.use_var(dividend_var);
        let divisor_val = self.builder.use_var(divisor_var);
        let dividend_32 = self.builder.ins().ireduce(types::I32, dividend_val);
        let divisor_32 = self.builder.ins().ireduce(types::I32, divisor_val);

        // Check for division by zero - return dividend if divisor is zero
        let zero = self.builder.ins().iconst(types::I32, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor_32, zero);

        let one_32 = self.builder.ins().iconst(types::I32, 1);
        let safe_divisor = self.builder.ins().select(is_zero, one_32, divisor_32);
        let result_32 = self.builder.ins().urem(dividend_32, safe_divisor);
        let result_32_ext = self.builder.ins().sextend(types::I64, result_32);

        // Return original dividend for div by zero, otherwise remainder
        let dividend_32_ext = self.builder.ins().sextend(types::I64, dividend_32);
        let result = self
            .builder
            .ins()
            .select(is_zero, dividend_32_ext, result_32_ext);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_rem_u_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend_var = self.registers[&reg0];
        let divisor_var = self.registers[&reg1];
        let dividend_val = self.builder.use_var(dividend_var);
        let divisor_val = self.builder.use_var(divisor_var);

        // Check for division by zero - return dividend if divisor is zero
        let zero = self.builder.ins().iconst(types::I64, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor_val, zero);

        let one_64 = self.builder.ins().iconst(types::I64, 1);
        let safe_divisor = self.builder.ins().select(is_zero, one_64, divisor_val);
        let result_rem = self.builder.ins().urem(dividend_val, safe_divisor);

        // Return original dividend for div by zero, otherwise remainder
        let result = self.builder.ins().select(is_zero, dividend_val, result_rem);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_rem_s_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend_var = self.registers[&reg0];
        let divisor_var = self.registers[&reg1];
        let dividend_val = self.builder.use_var(dividend_var);
        let divisor_val = self.builder.use_var(divisor_var);
        let dividend_32 = self.builder.ins().ireduce(types::I32, dividend_val);
        let divisor_32 = self.builder.ins().ireduce(types::I32, divisor_val);

        // Check for division by zero
        let zero = self.builder.ins().iconst(types::I32, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor_32, zero);

        // Check for overflow (i32::MIN % -1)
        let min_val_32 = self.builder.ins().iconst(types::I32, i32::MIN as i64);
        let neg_one = self.builder.ins().iconst(types::I32, -1);
        let is_min = self
            .builder
            .ins()
            .icmp(IntCC::Equal, dividend_32, min_val_32);
        let is_neg_one = self.builder.ins().icmp(IntCC::Equal, divisor_32, neg_one);
        let is_overflow = self.builder.ins().band(is_min, is_neg_one);

        // Use safe divisor to avoid division faults
        let one_32 = self.builder.ins().iconst(types::I32, 1);
        let safe_divisor = self.builder.ins().select(is_zero, one_32, divisor_32);
        let safe_divisor = self.builder.ins().select(is_overflow, one_32, safe_divisor);
        let result_32 = self.builder.ins().srem(dividend_32, safe_divisor);
        let result_32_ext = self.builder.ins().sextend(types::I64, result_32);

        // Return original dividend for div by zero, 0 for overflow, otherwise remainder
        let dividend_32_ext = self.builder.ins().sextend(types::I64, dividend_32);
        let zero_64 = self.builder.ins().iconst(types::I64, 0);
        let result_or_overflow = self
            .builder
            .ins()
            .select(is_overflow, zero_64, result_32_ext);
        let result = self
            .builder
            .ins()
            .select(is_zero, dividend_32_ext, result_or_overflow);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_rem_s_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend_var = self.registers[&reg0];
        let divisor_var = self.registers[&reg1];
        let dividend_val = self.builder.use_var(dividend_var);
        let divisor_val = self.builder.use_var(divisor_var);

        // Check for division by zero
        let zero = self.builder.ins().iconst(types::I64, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor_val, zero);

        // Check for overflow (i64::MIN % -1)
        let min_val_64 = self.builder.ins().iconst(types::I64, i64::MIN);
        let neg_one = self.builder.ins().iconst(types::I64, -1);
        let is_min = self
            .builder
            .ins()
            .icmp(IntCC::Equal, dividend_val, min_val_64);
        let is_neg_one = self.builder.ins().icmp(IntCC::Equal, divisor_val, neg_one);
        let is_overflow = self.builder.ins().band(is_min, is_neg_one);

        // Use safe divisor to avoid division faults
        let one_64 = self.builder.ins().iconst(types::I64, 1);
        let safe_divisor = self.builder.ins().select(is_zero, one_64, divisor_val);
        let safe_divisor = self.builder.ins().select(is_overflow, one_64, safe_divisor);
        let result_rem = self.builder.ins().srem(dividend_val, safe_divisor);

        // Return original dividend for div by zero, 0 for overflow, otherwise remainder
        let zero_64 = self.builder.ins().iconst(types::I64, 0);
        let result_or_overflow = self.builder.ins().select(is_overflow, zero_64, result_rem);
        let result = self
            .builder
            .ins()
            .select(is_zero, dividend_val, result_or_overflow);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_shlo_l_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let src0_32 = self.builder.ins().ireduce(types::I32, src0_val);
        let shift_val = self.builder.ins().ireduce(types::I32, src1_val);

        // Mask shift amount to avoid undefined behavior
        let mask = self.builder.ins().iconst(types::I32, 31);
        let safe_shift = self.builder.ins().band(shift_val, mask);
        let result_32 = self.builder.ins().ishl(src0_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_shlo_l_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);

        // Mask shift amount to avoid undefined behavior
        let mask = self.builder.ins().iconst(types::I64, 63);
        let safe_shift = self.builder.ins().band(src1_val, mask);
        let result = self.builder.ins().ishl(src0_val, safe_shift);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_shlo_l_imm_32(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);

        // Mask immediate to avoid undefined behavior
        let safe_shift = (imm0 % 32) as i64;
        let result_32 = self.builder.ins().ishl_imm(src_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_shlo_l_imm_64(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);

        // Mask immediate to avoid undefined behavior
        let safe_shift = (imm0 % 64) as i64;
        let result = self.builder.ins().ishl_imm(src_val, safe_shift);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_shlo_r_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let src0_32 = self.builder.ins().ireduce(types::I32, src0_val);
        let shift_val = self.builder.ins().ireduce(types::I32, src1_val);

        // Mask shift amount to avoid undefined behavior
        let mask = self.builder.ins().iconst(types::I32, 31);
        let safe_shift = self.builder.ins().band(shift_val, mask);
        let result_32 = self.builder.ins().ushr(src0_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_shlo_r_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);

        // Mask shift amount to avoid undefined behavior
        let mask = self.builder.ins().iconst(types::I64, 63);
        let safe_shift = self.builder.ins().band(src1_val, mask);
        let result = self.builder.ins().ushr(src0_val, safe_shift);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_shlo_r_imm_32(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);

        // Mask immediate to avoid undefined behavior
        let safe_shift = (imm0 % 32) as i64;
        let result_32 = self.builder.ins().ushr_imm(src_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_shlo_r_imm_64(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);

        // Mask immediate to avoid undefined behavior
        let safe_shift = (imm0 % 64) as i64;
        let result = self.builder.ins().ushr_imm(src_val, safe_shift);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_shar_r_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let src0_32 = self.builder.ins().ireduce(types::I32, src0_val);
        let shift_val = self.builder.ins().ireduce(types::I32, src1_val);

        // Mask shift amount to avoid undefined behavior
        let mask = self.builder.ins().iconst(types::I32, 31);
        let safe_shift = self.builder.ins().band(shift_val, mask);
        let result_32 = self.builder.ins().sshr(src0_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_shar_r_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);

        // Mask shift amount to avoid undefined behavior
        let mask = self.builder.ins().iconst(types::I64, 63);
        let safe_shift = self.builder.ins().band(src1_val, mask);
        let result = self.builder.ins().sshr(src0_val, safe_shift);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_shar_r_imm_32(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);

        // Mask immediate to avoid undefined behavior
        let safe_shift = (imm0 % 32) as i64;
        let result_32 = self.builder.ins().sshr_imm(src_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_shar_r_imm_64(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);

        // Mask immediate to avoid undefined behavior
        let safe_shift = (imm0 % 64) as i64;
        let result = self.builder.ins().sshr_imm(src_val, safe_shift);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    // Bit counting operations
    fn visit_leading_zero_bits_64(&mut self, format: format::RR) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);

        let result = self.builder.ins().clz(src_val);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_leading_zero_bits_32(&mut self, format: format::RR) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);

        let result_32 = self.builder.ins().clz(src_32);
        let result_64 = self.builder.ins().uextend(types::I64, result_32);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_trailing_zero_bits_64(&mut self, format: format::RR) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);

        let result = self.builder.ins().ctz(src_val);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_trailing_zero_bits_32(&mut self, format: format::RR) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);

        let result_32 = self.builder.ins().ctz(src_32);
        let result_64 = self.builder.ins().uextend(types::I64, result_32);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    // Rotation operations - register variants
    fn visit_rot_l_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src_var = self.registers[&reg0];
        let shift_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let shift_val = self.builder.use_var(shift_var);

        // Mask shift amount to avoid undefined behavior
        let mask = self.builder.ins().iconst(types::I64, 63);
        let safe_shift = self.builder.ins().band(shift_val, mask);
        let result = self.builder.ins().rotl(src_val, safe_shift);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_rot_l_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src_var = self.registers[&reg0];
        let shift_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let shift_val = self.builder.use_var(shift_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        let shift_32 = self.builder.ins().ireduce(types::I32, shift_val);

        // Mask shift amount to avoid undefined behavior
        let mask = self.builder.ins().iconst(types::I32, 31);
        let safe_shift = self.builder.ins().band(shift_32, mask);
        let result_32 = self.builder.ins().rotl(src_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_rot_r_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src_var = self.registers[&reg0];
        let shift_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let shift_val = self.builder.use_var(shift_var);

        // Mask shift amount to avoid undefined behavior
        let mask = self.builder.ins().iconst(types::I64, 63);
        let safe_shift = self.builder.ins().band(shift_val, mask);
        let result = self.builder.ins().rotr(src_val, safe_shift);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_rot_r_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src_var = self.registers[&reg0];
        let shift_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let shift_val = self.builder.use_var(shift_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        let shift_32 = self.builder.ins().ireduce(types::I32, shift_val);

        // Mask shift amount to avoid undefined behavior
        let mask = self.builder.ins().iconst(types::I32, 31);
        let safe_shift = self.builder.ins().band(shift_32, mask);
        let result_32 = self.builder.ins().rotr(src_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    // Rotation operations - immediate variants
    fn visit_rot_r_64_imm(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);

        // Mask immediate to avoid undefined behavior
        let safe_shift = (imm0 % 64) as i64;
        let result = self.builder.ins().rotr_imm(src_val, safe_shift);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_rot_r_32_imm(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);

        // Mask immediate to avoid undefined behavior
        let safe_shift = (imm0 % 32) as i64;
        let result_32 = self.builder.ins().rotr_imm(src_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    // Memory load operations
    fn visit_load_u8(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let dst_var = self.registers[&reg0];

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Emit memory read
        let value = self.emit_memory_read(address, MemorySize::Byte);
        let extended = self.builder.ins().uextend(types::I64, value);

        self.builder.def_var(dst_var, extended);
        Ok(())
    }

    fn visit_load_u16(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let dst_var = self.registers[&reg0];

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Emit memory read
        let value = self.emit_memory_read(address, MemorySize::Word);
        let extended = self.builder.ins().uextend(types::I64, value);

        self.builder.def_var(dst_var, extended);
        Ok(())
    }

    fn visit_load_u32(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let dst_var = self.registers[&reg0];

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Emit memory read
        let value = self.emit_memory_read(address, MemorySize::DWord);
        let extended = self.builder.ins().uextend(types::I64, value);

        self.builder.def_var(dst_var, extended);
        Ok(())
    }

    fn visit_load_u64(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let dst_var = self.registers[&reg0];

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Emit memory read
        let value = self.emit_memory_read(address, MemorySize::QWord);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    fn visit_load_i8(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let dst_var = self.registers[&reg0];

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Emit signed memory read
        let value = self.emit_memory_read_signed(address, MemorySize::Byte);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    fn visit_load_i16(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let dst_var = self.registers[&reg0];

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Emit signed memory read
        let value = self.emit_memory_read_signed(address, MemorySize::Word);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    fn visit_load_i32(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let dst_var = self.registers[&reg0];

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Emit signed memory read
        let value = self.emit_memory_read_signed(address, MemorySize::DWord);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    // Indirect load operations
    fn visit_load_ind_u8(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let dst_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit memory read
        let value = self.emit_memory_read(effective_addr, MemorySize::Byte);
        let extended = self.builder.ins().uextend(types::I64, value);

        self.builder.def_var(dst_var, extended);
        Ok(())
    }

    fn visit_load_ind_u16(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let dst_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit memory read
        let value = self.emit_memory_read(effective_addr, MemorySize::Word);
        let extended = self.builder.ins().uextend(types::I64, value);

        self.builder.def_var(dst_var, extended);
        Ok(())
    }

    fn visit_load_ind_u32(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let dst_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit memory read
        let value = self.emit_memory_read(effective_addr, MemorySize::DWord);
        let extended = self.builder.ins().uextend(types::I64, value);

        self.builder.def_var(dst_var, extended);
        Ok(())
    }

    fn visit_load_ind_u64(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let dst_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit memory read
        let value = self.emit_memory_read(effective_addr, MemorySize::QWord);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    fn visit_load_ind_i8(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let dst_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit signed memory read
        let value = self.emit_memory_read_signed(effective_addr, MemorySize::Byte);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    fn visit_load_ind_i16(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let dst_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit signed memory read
        let value = self.emit_memory_read_signed(effective_addr, MemorySize::Word);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    fn visit_load_ind_i32(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let dst_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit signed memory read
        let value = self.emit_memory_read_signed(effective_addr, MemorySize::DWord);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    // Branch operations - for linear execution, they're no-ops that terminate blocks
    fn visit_branch_eq(&mut self, _format: format::RRO) -> Result<(), Self::Error> {
        // For linear execution, branches are no-ops but we need to end the block
        // Add a fallthrough instruction to satisfy Cranelift's terminator requirement
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_eq_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_ne(&mut self, _format: format::RRO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_ne_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_lt_u(&mut self, _format: format::RRO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_lt_s(&mut self, _format: format::RRO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_ge_u(&mut self, _format: format::RRO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_ge_s(&mut self, _format: format::RRO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_lt_u_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_lt_s_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_ge_u_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_ge_s_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    // Additional branch operations
    fn visit_branch_gt_u_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_gt_s_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_le_u_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_le_s_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    // Jump operations
    fn visit_jump(&mut self, _format: format::O) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_jump_ind(&mut self, _format: format::RI) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    // Conditional move operations
    fn visit_cmov_iz(&mut self, _format: format::RRR) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_cmov_iz_imm(&mut self, _format: format::RRI) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_cmov_nz(&mut self, _format: format::RRR) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_cmov_nz_imm(&mut self, _format: format::RRI) -> Result<(), Self::Error> {
        Ok(())
    }

    // Load immediate and jump indirect operations
    fn visit_load_imm_jump_ind(&mut self, _format: format::RRII) -> Result<(), Self::Error> {
        Ok(())
    }

    // Store operations
    fn visit_store_u8(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let src_var = self.registers[&reg0];
        let src_val = self.builder.use_var(src_var);

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Truncate to 8 bits
        let truncated = self.builder.ins().ireduce(types::I8, src_val);

        // Emit memory write
        self.emit_memory_write(address, truncated, MemorySize::Byte);

        Ok(())
    }

    fn visit_store_u16(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let src_var = self.registers[&reg0];
        let src_val = self.builder.use_var(src_var);

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Truncate to 16 bits
        let truncated = self.builder.ins().ireduce(types::I16, src_val);

        // Emit memory write
        self.emit_memory_write(address, truncated, MemorySize::Word);

        Ok(())
    }

    fn visit_store_u32(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let src_var = self.registers[&reg0];
        let src_val = self.builder.use_var(src_var);

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Truncate to 32 bits
        let truncated = self.builder.ins().ireduce(types::I32, src_val);

        // Emit memory write
        self.emit_memory_write(address, truncated, MemorySize::DWord);

        Ok(())
    }

    fn visit_store_u64(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let src_var = self.registers[&reg0];
        let src_val = self.builder.use_var(src_var);

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Emit memory write
        self.emit_memory_write(address, src_val, MemorySize::QWord);

        Ok(())
    }

    // Store immediate operations
    fn visit_store_imm_u8(&mut self, format: format::II) -> Result<(), Self::Error> {
        let format::II { imm0, imm1 } = format;

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I8, (imm1 as u8) as i64);

        // Emit memory write
        self.emit_memory_write(address, value, MemorySize::Byte);

        Ok(())
    }

    fn visit_store_imm_u16(&mut self, format: format::II) -> Result<(), Self::Error> {
        let format::II { imm0, imm1 } = format;

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I16, (imm1 as u16) as i64);

        // Emit memory write
        self.emit_memory_write(address, value, MemorySize::Word);

        Ok(())
    }

    fn visit_store_imm_u32(&mut self, format: format::II) -> Result<(), Self::Error> {
        let format::II { imm0, imm1 } = format;

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I32, (imm1 as u32) as i64);

        // Emit memory write
        self.emit_memory_write(address, value, MemorySize::DWord);

        Ok(())
    }

    fn visit_store_imm_u64(&mut self, format: format::II) -> Result<(), Self::Error> {
        let format::II { imm0, imm1 } = format;

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I64, imm1 as i64);

        // Emit memory write
        self.emit_memory_write(address, value, MemorySize::QWord);

        Ok(())
    }

    // Indirect store operations
    fn visit_store_ind_u8(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Truncate to 8 bits
        let truncated = self.builder.ins().ireduce(types::I8, src_val);

        // Emit memory write
        self.emit_memory_write(effective_addr, truncated, MemorySize::Byte);

        Ok(())
    }

    fn visit_store_ind_u16(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Truncate to 16 bits
        let truncated = self.builder.ins().ireduce(types::I16, src_val);

        // Emit memory write
        self.emit_memory_write(effective_addr, truncated, MemorySize::Word);

        Ok(())
    }

    fn visit_store_ind_u32(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Truncate to 32 bits
        let truncated = self.builder.ins().ireduce(types::I32, src_val);

        // Emit memory write
        self.emit_memory_write(effective_addr, truncated, MemorySize::DWord);

        Ok(())
    }

    fn visit_store_ind_u64(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit memory write
        self.emit_memory_write(effective_addr, src_val, MemorySize::QWord);

        Ok(())
    }

    // Store immediate indirect operations
    fn visit_store_imm_ind_u8(&mut self, format: format::RII) -> Result<(), Self::Error> {
        let format::RII { reg0, imm0, imm1 } = format;
        let addr_var = self.registers[&reg0];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I8, (imm1 as u8) as i64);

        // Emit memory write
        self.emit_memory_write(effective_addr, value, MemorySize::Byte);

        Ok(())
    }

    fn visit_store_imm_ind_u16(&mut self, format: format::RII) -> Result<(), Self::Error> {
        let format::RII { reg0, imm0, imm1 } = format;
        let addr_var = self.registers[&reg0];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I16, (imm1 as u16) as i64);

        // Emit memory write
        self.emit_memory_write(effective_addr, value, MemorySize::Word);

        Ok(())
    }

    fn visit_store_imm_ind_u32(&mut self, format: format::RII) -> Result<(), Self::Error> {
        let format::RII { reg0, imm0, imm1 } = format;
        let addr_var = self.registers[&reg0];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I32, (imm1 as u32) as i64);

        // Emit memory write
        self.emit_memory_write(effective_addr, value, MemorySize::DWord);

        Ok(())
    }

    fn visit_store_imm_ind_u64(&mut self, format: format::RII) -> Result<(), Self::Error> {
        let format::RII { reg0, imm0, imm1 } = format;
        let addr_var = self.registers[&reg0];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I64, imm1 as i64);

        // Emit memory write
        self.emit_memory_write(effective_addr, value, MemorySize::QWord);

        Ok(())
    }
}
