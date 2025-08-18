//! translation utils

use crate::{constants::PVM_REGISTER_COUNT, translator::Block, Translator};
use anyhow::Result;
use cranelift::prelude::*;
use cranelift_codegen::ir;
use parser::Visitor;

impl Translator<'_> {
    /// Analyze program - discovers all basic blocks using read_block()
    /// Uses parser's natural block discovery for clean, efficient block creation
    pub fn analyze(&mut self, program: &[u8]) -> Result<bool> {
        let blob = parser::program::deblob(program)?;
        self.jump_table = blob.jump_table.clone();
        self.blocks.clear();

        let mut reader = blob.reader();
        let mut has_trap = false;

        // Use read_block() to naturally discover block boundaries
        while !reader.eof() {
            let block_start = reader.position;
            let block_instructions = reader.read_block()?;

            if block_instructions.is_empty() {
                break;
            }

            // Block terminates if it contains a terminating instruction OR if we reached EOF
            let terminates = !block_instructions.is_empty()
                && (reader.eof() || block_instructions.last().unwrap().value.is_termination());

            // Handle indirect jump table targets first
            self.process_jump_targets(&block_instructions, &blob)?;

            // Check all instructions in the block for trap instructions
            for instr in &block_instructions {
                if matches!(instr.value, parser::Instruction::Trap) {
                    has_trap = true;
                }
            }

            // Only create block if it doesn't already exist (might have been created by process_jump_targets)
            if !self.blocks.contains_key(&(block_start as u64)) {
                self.pvm_blocks.insert(
                    block_start as u64,
                    crate::Block {
                        start: block_start,
                        end: reader.position,
                        terminates,
                        instructions: block_instructions,
                    },
                );
            }
        }

        for pc in self.pvm_blocks.keys() {
            if !self.blocks.contains_key(pc) {
                self.blocks.insert(*pc, self.builder.create_block());
            }
        }

        Ok(has_trap)
    }

    /// Translate entire program
    pub fn translate(&mut self, entry: ir::Block) -> Result<()> {
        let ctx_ptr = self.builder.block_params(entry)[0];
        let start_pc = self.builder.block_params(entry)[1];
        self.init_with_context(ctx_ptr)?;

        // Load all registers from context ONCE at function entry
        for i in 0..PVM_REGISTER_COUNT {
            let reg_var = self.registers[&(i as u8)];
            let offset = self.builder.ins().iconst(types::I64, (i * 8) as i64);
            let addr = self.builder.ins().iadd(ctx_ptr, offset);
            let reg_val = self
                .builder
                .ins()
                .load(types::I64, MemFlags::new(), addr, 0);
            self.builder.def_var(reg_var, reg_val);
        }

        // Dispatcher: jump to the requested starting PC
        // Create a switch statement to jump to the correct block based on start_pc
        let mut switch = cranelift::frontend::Switch::new();
        for (&pc, &cranelift_block) in &self.blocks {
            switch.set_entry(pc as u128, cranelift_block);
        }

        // Default case: if PC is not found, return with trap
        let default_block = self.builder.create_block();
        self.builder.switch_to_block(default_block);
        self.return_trap()?;
        self.builder.seal_block(default_block);

        // Generate the switch on start_pc
        self.builder.switch_to_block(entry);
        switch.emit(&mut self.builder, start_pc, default_block);
        self.builder.seal_block(entry);

        // Step 2: Translate all PVM blocks to Cranelift basic blocks using shared translator
        //
        // TODO: remove the clone here.
        for (pc, pvm_block) in &self.pvm_blocks.clone() {
            let cranelift_block = self.blocks[pc];
            self.builder.switch_to_block(cranelift_block);

            // Translate instructions in this block using shared translator
            match self.translate_block(pvm_block) {
                Ok(_) => {
                    tracing::trace!("Successfully translated block at PC {}", pc);
                }
                Err(e) => {
                    tracing::warn!("Failed to translate block at PC {}: {}", pc, e);
                    // Generate trap for failed blocks
                    self.return_trap()?;
                }
            }
        }

        // Step 3: Seal all blocks after translation
        for &cranelift_block in self.blocks.values() {
            self.builder.seal_block(cranelift_block);
        }

        Ok(())
    }

    /// Translate block and check termination
    pub fn translate_block(&mut self, block: &Block) -> Result<()> {
        // Translate all instructions in this block
        for instruction in &block.instructions {
            let pc = instruction.range.start;
            tracing::trace!("translating PC {} instruction {:?}", pc, instruction.value);

            if let Err(e) = self.visit(instruction.value, &instruction.range) {
                tracing::warn!("Instruction translation failed at PC {}: {}", pc, e);
            }
        }

        // Handle block termination with native Cranelift control flow
        //
        // This is only for the tests that has incomplete blocks.
        if let Some(last) = block.instructions.last() {
            if !last.value.is_termination() {
                self.return_continue_with_pc(block.end as u64)?;
            }
        }
        Ok(())
    }

    /// Process jump targets from indirect jump instructions
    fn process_jump_targets(
        &mut self,
        block_instructions: &[parser::reader::Offset<parser::Instruction>],
        blob: &parser::program::ProgramBlob,
    ) -> Result<()> {
        let Some(last_instruction) = block_instructions.last() else {
            return Ok(());
        };

        if !matches!(
            last_instruction.value,
            parser::Instruction::JumpInd(_) | parser::Instruction::LoadImmJumpInd(_)
        ) {
            return Ok(());
        }

        // Clone jump_table to avoid borrow checker issues
        let jump_table = self.jump_table.clone();
        for &target in &jump_table {
            if (target as usize) >= blob.instructions.len() || self.blocks.contains_key(&target) {
                continue;
            }

            let mut target_reader = blob.reader();
            target_reader.set_position(target as usize);
            if target_reader.eof() {
                continue;
            }

            let target_start = target_reader.position;
            let target_instructions = target_reader.read_block()?;
            let target_end = target_reader.position;

            // Check if the block actually terminates (has a terminating instruction)
            let terminates = !target_instructions.is_empty()
                && target_instructions.last().unwrap().value.is_termination();

            self.pvm_blocks.insert(
                target,
                crate::translator::Block {
                    start: target_start,
                    end: target_end,
                    terminates,
                    instructions: target_instructions,
                },
            );
        }

        Ok(())
    }
}
