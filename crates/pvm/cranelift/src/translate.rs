//! translation utils

use crate::{Exit, Translator};
use anyhow::Result;
use cranelift::prelude::InstBuilder;
use cranelift_codegen::ir::{self, BlockCall};
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
        self.jump = blob.jump_table.clone();
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
    ///
    /// FIXME: now is using a hard coded 5 as the starting PC for accumulate programs
    /// this should be fixed after fixing our 0.7.0 reports.
    fn translate_dispatcher(&mut self, program: &Program, entry: ir::Block) -> Result<()> {
        let ctx_ptr = self.builder.block_params(entry)[0];
        self.builder.switch_to_block(entry);
        self.init_context(program, ctx_ptr);

        // Generate the runtime jump table for djump instructions
        let trap_block = self.builder.create_block();
        let default_block = BlockCall::new(
            trap_block,
            std::iter::empty(),
            &mut self.builder.func.dfg.value_lists,
        );

        // Create block calls for each jump table entry
        let mut block_calls = Vec::with_capacity(self.jump.len());
        for &jump_pc in &self.jump {
            let block = self.blocks.get(&jump_pc).copied().unwrap_or(trap_block);
            let block_call = BlockCall::new(
                block,
                std::iter::empty(),
                &mut self.builder.func.dfg.value_lists,
            );
            block_calls.push(block_call);
        }

        // Create and cache the jump table
        let jt_data = cranelift_codegen::ir::JumpTableData::new(default_block, &block_calls);
        self.rt_jump_table = self.builder.create_jump_table(jt_data);
        self.builder.switch_to_block(trap_block);
        self.return_(Exit::InvalidJumpTarget);
        self.builder.seal_block(trap_block);

        // Now handle the entry point - hardcoded to 5 for now
        self.builder.switch_to_block(entry);
        if let Some(&start_block) = self.blocks.get(&5) {
            self.builder.ins().jump(start_block, &[]);
        } else {
            self.return_(Exit::InvalidStartPC);
        }

        self.builder.seal_block(entry);
        Ok(())
    }

    /// translate a block and check termination
    fn translate_block(&mut self, block: &Block) -> Result<()> {
        for instruction in block {
            // tracing::trace!("{instruction:?}");
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
                self.return_(Exit::ProgramNotTerminated);
            }
        }

        Ok(())
    }
}
