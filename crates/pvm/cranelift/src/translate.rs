//! translation utils

use crate::{control::result, Translator};
use anyhow::Result;
use cranelift_codegen::ir;
use parser::{reader::Offset, Instruction, Visitor};
use pvm::Program;
use std::collections::BTreeMap;

type Block = Vec<Offset<Instruction>>;

impl Translator<'_> {
    /// Translate entire program
    pub fn translate(&mut self, program: &Program) -> Result<()> {
        let entry = self.builder.create_block();
        self.builder.append_block_params_for_function_params(entry);

        // analyze blocks
        let blocks = self.analyze(&program.code)?;
        self.translate_dispatcher(program, entry)?;
        for (pc, block) in &blocks {
            let cblock = self.blocks[pc];
            self.builder.switch_to_block(cblock);
            self.burn_gas(block.len() as i64);
            self.translate_block(block)?;
        }

        self.builder.seal_all_blocks();
        Ok(())
    }

    /// discovers all basic blocks
    fn analyze(&mut self, program: &[u8]) -> Result<BTreeMap<u64, Block>> {
        let blob = parser::program::deblob(program)?;
        self.jump_table = blob.jump_table.clone();
        let mut reader = blob.reader();
        let mut blocks = BTreeMap::new();
        while !reader.eof() {
            let block_start = reader.position;
            let instructions = reader.read_block()?;
            if instructions.is_empty() {
                break;
            }

            blocks.insert(block_start as u64, instructions);
            self.blocks
                .insert(block_start as u64, self.builder.create_block());
        }

        Ok(blocks)
    }

    /// translate the dispatcher
    fn translate_dispatcher(&mut self, program: &Program, entry: ir::Block) -> Result<()> {
        let trap = self.builder.create_block();
        let ctx_ptr = self.builder.block_params(entry)[0];
        let mut switch = cranelift::frontend::Switch::new();
        for (&pc, &block) in &self.blocks {
            switch.set_entry(pc as u128, block);
        }

        // generate the switch on start_pc
        self.builder.switch_to_block(entry);
        self.init_context(program, ctx_ptr);
        let pc = self.pc();
        switch.emit(&mut self.builder, pc, trap);
        self.builder.seal_block(entry);

        // populate trap block
        self.builder.switch_to_block(trap);
        self.return_(result::PANIC, 0)?;
        self.builder.seal_block(trap);
        Ok(())
    }

    /// translate a block and check termination
    fn translate_block(&mut self, block: &Block) -> Result<()> {
        for instruction in block {
            tracing::trace!("{instruction:?}");
            if let Err(e) = self.visit(instruction.value, &instruction.range) {
                tracing::warn!(
                    "Instruction translation failed at PC {}: {}",
                    instruction.range.start,
                    e
                );
            }
        }

        // handle block termination with native CLIF control flow
        if let Some(last) = block.last() {
            if !last.value.is_termination() {
                self.burn_gas(1);
                self.return_(result::PANIC, last.range.end)?;
            }
        }

        Ok(())
    }
}
