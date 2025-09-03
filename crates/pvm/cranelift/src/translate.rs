//! translation utils

use crate::{Exit, Translator};
use anyhow::Result;
use cranelift::prelude::{types, InstBuilder, IntCC, JumpTableData};
use cranelift_codegen::ir::{self, BlockArg, BlockCall};
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
            if self.need_sync(pc) {
                self.load_registers();
            } else {
                let params = (0..13)
                    .map(|_| self.builder.append_block_param(cblock, types::I64))
                    .collect::<Vec<_>>();
                self.params(&params);
            }
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
    fn translate_dispatcher(&mut self, program: &Program, entry: ir::Block) -> Result<()> {
        let ctx_ptr = self.builder.block_params(entry)[0];
        self.builder.switch_to_block(entry);
        self.init_context(program, ctx_ptr);

        // Generate the runtime jump table for djump instructions
        let trap = self.builder.create_block();
        let default_block = BlockCall::new(
            trap,
            std::iter::empty(),
            &mut self.builder.func.dfg.value_lists,
        );

        // Create block calls pointing directly to target blocks (no adapters needed)
        let mut block_calls = Vec::with_capacity(self.jump.len());
        for &jump_pc in &self.jump {
            let target = self.blocks.get(&jump_pc).copied().unwrap_or(trap);
            let block_call = BlockCall::new(
                target,
                std::iter::empty(),
                &mut self.builder.func.dfg.value_lists,
            );
            block_calls.push(block_call);
        }

        // Create and cache the jump table
        let jt_data = JumpTableData::new(default_block, &block_calls);
        self.rt_jump_table = self.builder.create_jump_table(jt_data);
        self.builder.switch_to_block(trap);
        self.return_(Exit::InvalidJumpTarget);
        self.builder.seal_block(trap);
        self.translate_entry(entry, trap)?;
        Ok(())
    }

    /// Handle the entry point
    ///
    /// - 5 for accumulate programs
    /// - 13 for testing
    /// - 0 for general programs
    fn translate_entry(&mut self, entry: ir::Block, trap: ir::Block) -> Result<()> {
        self.builder.switch_to_block(entry);
        let pc = self.pc();

        // Check for pc == 5 (accumulate)
        let five = self.builder.ins().iconst(types::I64, 5);
        let is_accumulate = self.builder.ins().icmp(IntCC::Equal, pc, five);
        let accumulate = self.blocks.get(&5).copied().unwrap_or(trap);

        // Check for pc == 13 (testing)
        let thirteen = self.builder.ins().iconst(types::I64, 13);
        let is_test = self.builder.ins().icmp(IntCC::Equal, pc, thirteen);
        let test = self.blocks.get(&13).copied().unwrap_or(trap);

        // Default to block 0 (general)
        let general = self.blocks.get(&0).copied().unwrap_or(trap);

        // construct the arguments for the blocks
        self.load_registers();
        let args = self.args();
        let empty_args: Vec<BlockArg> = vec![];
        let [accumulate_args, test_args, general_args] = [accumulate, test, general].map(|b| {
            if b == trap {
                &empty_args[..]
            } else {
                &args[..]
            }
        });

        // Branch: if pc == 5 goto accumulate, else check for pc == 13
        let check_test = self.builder.create_block();
        {
            for _ in 0..13 {
                self.builder.append_block_param(check_test, types::I64);
            }

            self.builder.ins().brif(
                is_accumulate,
                accumulate,
                accumulate_args,
                check_test,
                &args,
            );
        }

        // Branch: if pc == 13 goto test, else goto general
        {
            self.builder.switch_to_block(check_test);
            let check_test_params = (0..13)
                .map(|i| self.builder.block_params(check_test)[i])
                .collect::<Vec<_>>();
            self.params(&check_test_params);
            self.builder
                .ins()
                .brif(is_test, test, test_args, general, general_args);
            self.builder.seal_block(check_test);
            self.builder.seal_block(entry);
        }
        Ok(())
    }

    /// translate a block and check termination
    fn translate_block(&mut self, block: &Block) -> Result<()> {
        for instruction in block {
            self.burn_gas(self.pool.one);
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
                self.burn_gas(self.pool.one);
                self.return_(Exit::ProgramNotTerminated);
            }
        }

        Ok(())
    }
}
