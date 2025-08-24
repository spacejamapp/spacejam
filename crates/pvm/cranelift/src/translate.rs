//! translation utils

use crate::Translator;
use anyhow::Result;
use parser::{reader::Offset, Instruction, Visitor};
use pvm::Program;
use std::collections::BTreeMap;

type Block = Vec<Offset<Instruction>>;

impl Translator<'_> {
    /// Translate entire program
    pub fn translate(&mut self, program: &Program) -> Result<bool> {
        let (has_trap, blocks) = self.analyze(&program.code)?;
        self.translate_dispatcher(program)?;
        for (pc, block) in &blocks {
            let cranelift_block = self.blocks[pc];
            self.builder.switch_to_block(cranelift_block);
            if let Err(e) = self.translate_block(block) {
                tracing::warn!("Failed to translate block at PC {}: {}", pc, e);
                self.return_trap()?;
            }
        }

        self.builder.seal_all_blocks();
        Ok(has_trap)
    }

    /// discovers all basic blocks
    fn analyze(&mut self, program: &[u8]) -> Result<(bool, BTreeMap<u64, Block>)> {
        let blob = parser::program::deblob(program)?;

        // read all blocks and create CLIF blocks
        let mut reader = blob.reader();
        let mut has_trap = false;
        let mut blocks = BTreeMap::new();
        while !reader.eof() {
            let block_start = reader.position;
            let instructions = reader.read_block()?;
            if instructions.is_empty() {
                break;
            }

            // check for trap instructions
            for instr in &instructions {
                if matches!(instr.value, parser::Instruction::Trap) {
                    has_trap = true;
                }
            }

            // Store the block
            blocks.insert(block_start as u64, instructions);
            self.blocks
                .insert(block_start as u64, self.builder.create_block());
        }

        Ok((has_trap, blocks))
    }

    /// translate the dispatcher
    fn translate_dispatcher(&mut self, program: &Program) -> Result<()> {
        let entry = self.builder.create_block();
        self.builder.append_block_params_for_function_params(entry);

        // get the context ptr
        let ctx_ptr = self.builder.block_params(entry)[0];
        self.init_context(program, ctx_ptr);

        // create a switch statement to jump to the correct block based on pc
        let mut switch = cranelift::frontend::Switch::new();
        for (&pc, &block) in &self.blocks {
            switch.set_entry(pc as u128, block);
        }

        // if the PC is not found, return with trap
        let default_block = self.builder.create_block();
        self.builder.switch_to_block(default_block);
        self.return_trap()?;
        self.builder.seal_block(default_block);

        // generate the switch on start_pc
        self.builder.switch_to_block(entry);
        let pc = self.pc();
        switch.emit(&mut self.builder, pc, default_block);
        self.builder.seal_block(entry);
        Ok(())
    }

    /// translate a block and check termination
    fn translate_block(&mut self, block: &Block) -> Result<()> {
        for instruction in block {
            let pc = instruction.range.start;
            tracing::trace!("translating PC {} instruction {:?}", pc, instruction.value);

            if let Err(e) = self.visit(instruction.value, &instruction.range) {
                tracing::warn!("Instruction translation failed at PC {}: {}", pc, e);
            }
        }

        // handle block termination with native CLIF control flow
        //
        // this is only for the tests that has incomplete blocks.
        if let Some(last) = block.last() {
            if !last.value.is_termination() {
                self.return_continue_with_pc(last.range.end as u64)?;
            }
        }

        Ok(())
    }
}
