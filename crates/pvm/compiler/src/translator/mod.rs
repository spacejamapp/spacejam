//! Translator module that converts PVM instructions to Cranelift IR

use cranelift::prelude::*;
use parser::{Instruction, Visitor};
use std::collections::{BTreeMap, BTreeSet, HashMap};

mod control;
mod memory;
mod visitor;

/// PVM-to-Cranelift translator
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
    jump_table_map: BTreeMap<u32, usize>, // Jump table address -> PC target mapping
    
    // Trap detection
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
            jump_table_map: BTreeMap::new(),
            has_explicit_trap: false,
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
    pub fn translate(&mut self, program: &[u8]) -> Result<(Vec<Value>, Value, bool), anyhow::Error> {
        let blob = parser::program::deblob(program)?;
        let mut reader = blob.reader();

        // For simple programs without branches, use linear execution
        if self.has_control_flow(&blob)? {
            // Create dummy value before we start control flow generation
            let dummy_val = self.builder.ins().iconst(types::I32, 0);

            // Pass 1: Analyze control flow to identify basic block boundaries
            self.analyze_control_flow(&blob)?;

            // Pass 2: Generate code with proper control flow
            self.generate_code(&blob)?;

            // Control flow handled everything including return - return dummy values
            Ok((vec![], dummy_val, self.has_explicit_trap))
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

            // Return all 13 register values + PC (the JIT will handle the return instruction)
            let mut register_values = Vec::with_capacity(13);
            for i in 0..13 {
                let var = self.registers[&(i as u8)];
                register_values.push(self.builder.use_var(var));
            }

            let pc_value = self.builder.use_var(self.pc);

            Ok((register_values, pc_value, self.has_explicit_trap))
        }
    }

    /// Pass 2: Generate Cranelift IR with proper basic blocks
    fn generate_code(&mut self, blob: &parser::program::ProgramBlob) -> Result<(), anyhow::Error> {
        // Create Cranelift blocks for all branch targets
        for &target_offset in &self.branch_targets {
            let block = self.builder.create_block();
            self.basic_blocks.insert(target_offset, block);
        }
        
        // Build jump table mapping: map addresses to PC targets 
        // The jump table contains PC values (indices into instruction data)
        // When a jump_indirect is executed, it computes an address and looks it up
        // The address protocol is: index = (address / 2) - 1, so address = (index + 1) * 2
        for (index, &pc_target) in blob.jump_table.iter().enumerate() {
            // Convert jump table index to the address that would be used to access it
            let address = ((index + 1) * 2) as u32;
            
            // Map the address to the PC target
            self.jump_table_map.insert(address, pc_target as usize);
        }

        // Create exit block for function termination
        let exit_block = self.builder.create_block();

        // Jump from the current block (JIT entry block) to our entry block (offset 0)
        let our_entry_block = self.basic_blocks[&0];
        // TODO: Fix context parameter passing later
        self.builder.ins().jump(our_entry_block, &[]);

        // Start with our entry block (offset 0)
        self.builder.switch_to_block(our_entry_block);

        let mut reader = blob.reader();
        let mut current_block_start = 0;
        let mut needs_fallthrough = true;

        while !reader.eof() {
            let instruction_offset = reader.read()?;
            let current_pc = instruction_offset.range.start;
            let instruction = instruction_offset.value;

            // Check if we need to switch to a new basic block
            if current_pc != current_block_start && self.branch_targets.contains(&current_pc) {
                // Seal the previous block since we're done with it
                let prev_block = self.basic_blocks[&current_block_start];
                self.builder.seal_block(prev_block);

                // Add fallthrough jump if the previous block didn't terminate
                if needs_fallthrough {
                    self.builder.ins().jump(exit_block, &[]);
                }

                // We've reached a new basic block target - switch to it
                let target_block = self.basic_blocks[&current_pc];
                self.builder.switch_to_block(target_block);

                // Set PC for this block (entry block or branch target)
                // Fallthrough blocks from branches already have PC set
                let target_pc = self.builder.ins().iconst(types::I64, current_pc as i64);
                self.builder.def_var(self.pc, target_pc);
                current_block_start = current_pc;
            }

            // Check if this is a branch/jump instruction
            let is_control_flow = matches!(
                instruction,
                Instruction::Trap
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
                    | Instruction::BranchGtSImm(_)
                    | Instruction::Jump(_)
                    | Instruction::JumpInd(_)
                    | Instruction::LoadImmJump(_)
                    | Instruction::LoadImmJumpInd(_)
            );

            // Increment PC for non-control-flow instructions
            // We always increment PC unless it's a control flow instruction
            if !is_control_flow {
                let current_pc_val = self.builder.use_var(self.pc);
                let instruction_size = self
                    .builder
                    .ins()
                    .iconst(types::I64, instruction_offset.range.len() as i64);
                let new_pc = self.builder.ins().iadd(current_pc_val, instruction_size);
                self.builder.def_var(self.pc, new_pc);
            }

            // Execute the instruction
            needs_fallthrough = self.visit_with_control_flow(
                instruction,
                current_pc,
                instruction_offset.range.end,
            )?;
        }

        // Seal the final block since we're done with it
        let final_block = self.basic_blocks[&current_block_start];
        self.builder.seal_block(final_block);

        // If the last block needs fallthrough, jump to exit
        if needs_fallthrough {
            self.builder.ins().jump(exit_block, &[]);
        }


        // Switch to exit block and save state to context before returning
        self.builder.switch_to_block(exit_block);
        self.builder.seal_block(exit_block);

        // Get the context pointer parameter from entry block
        let entry_block = self.builder.func.layout.blocks().nth(0).unwrap();
        let context_ptr = self.builder.block_params(entry_block)[0];

        // Store all 13 register values back to context.registers
        for i in 0..13 {
            let reg_var = self.registers[&(i as u8)];
            let reg_value = self.builder.use_var(reg_var);
            let offset = self.builder.ins().iconst(types::I64, (i * 8) as i64);
            let addr = self.builder.ins().iadd(context_ptr, offset);
            self.builder
                .ins()
                .store(MemFlags::new(), reg_value, addr, 0);
        }

        // Store PC back to context.pc (offset 104)
        let pc_value = self.builder.use_var(self.pc);
        let pc_offset = self.builder.ins().iconst(types::I64, 104);
        let pc_addr = self.builder.ins().iadd(context_ptr, pc_offset);
        self.builder
            .ins()
            .store(MemFlags::new(), pc_value, pc_addr, 0);

        self.builder.ins().return_(&[]);

        Ok(())
    }
}
