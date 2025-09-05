//! Translator API V2

use crate::{ir, Exit, Translator};
use anyhow::Result;
use cranelift::prelude::{types, Block, InstBuilder, IntCC};
use parser::{reader::Offset, Instruction};
use pvm::{MemoryInfo, Visitor};
use std::collections::BTreeMap;

const ACCUMULATE_PC: u64 = 5;
const REFINE_PC: u64 = 0;
const TEST_PC: u64 = 13;

impl Translator<'_> {
    /// Translate a regular PVM function (non-main)
    ///
    /// NOTE: we don't have any arguments here since are in the main function.
    pub fn translate(&mut self, fun: &ir::Function, _info: MemoryInfo) -> Result<()> {
        // create all blocks
        for pc in fun.blocks.keys() {
            let block = self.create_block();
            self.blocks.insert(*pc, block);
        }

        // translate the all blocks
        for (pc, block) in self.blocks.clone() {
            let instructions = &fun.blocks[&pc];
            self.builder.switch_to_block(block);
            self.load_block_args(block);
            self.translate_block(instructions)?;
        }

        self.builder.seal_all_blocks();
        Ok(())
    }

    /// Translate the main function
    ///
    /// We need to init all block parameters to our registers here.
    ///
    /// [ctx, memory, gas, [..registers]]
    pub fn translate_main(&mut self, func: &ir::Function, _info: MemoryInfo) -> Result<()> {
        let entry = self.builder.create_block();
        self.builder.append_block_params_for_function_params(entry);
        self.builder.switch_to_block(entry);

        // init all registers from block parameters
        let params = self.builder.block_params(entry);
        let [memory, pc, gas] = [params[0], params[1], params[2]];
        (self.pool.memory, self.pool.gas) = (memory, gas);
        self.pool.registers = [
            params[3], params[4], params[5], params[6], params[7], params[8], params[9],
            params[10], params[11], params[12], params[13], params[14], params[15],
        ];

        // create all blocks
        for pc in func.blocks.keys() {
            let block = self.create_block();
            self.blocks.insert(*pc, block);
        }

        // Add the initial minimal dispatch logic in the entry block
        {
            let five = self.builder.ins().iconst(types::I8, ACCUMULATE_PC as i64);
            let thirteen = self.builder.ins().iconst(types::I8, TEST_PC as i64);
            let refine = self.blocks[&REFINE_PC];
            let accumulate = self.blocks.get(&ACCUMULATE_PC).cloned().unwrap_or(refine);
            let test = self.blocks.get(&TEST_PC).cloned().unwrap_or(refine);
            let check_test = self.create_block();

            // build the initial condition in the entry block
            let block_args = self.block_args();
            let is_accumulate = self.builder.ins().icmp(IntCC::Equal, pc, five);
            self.builder.ins().brif(
                is_accumulate,
                accumulate,
                &block_args,
                check_test,
                &block_args,
            );
            self.builder.seal_block(entry);

            // build the check test block
            self.builder.switch_to_block(check_test);
            self.load_block_args(check_test);
            let block_args = self.block_args();
            let is_test = self.builder.ins().icmp(IntCC::Equal, pc, thirteen);
            self.builder
                .ins()
                .brif(is_test, test, &block_args, refine, &block_args);
            self.builder.seal_block(check_test);
        }

        // now translate the rest of the blocks
        for (pc, block) in self.blocks.clone() {
            let instructions = &func.blocks[&pc];
            self.builder.switch_to_block(block);
            self.load_block_args(block);
            self.translate_block(instructions)?;
        }

        self.builder.seal_all_blocks();
        Ok(())
    }

    /// translate a block and check termination
    pub fn translate_block(&mut self, block: &[Offset<Instruction>]) -> Result<()> {
        let mut gas_map = BTreeMap::new();
        let mut gas = 0;
        for (index, instr) in block.iter().enumerate() {
            tracing::debug!("instr: {:?}", instr.value);
            if instr.value.is_memory_op() {
                gas_map.insert(index, gas - 1);
                gas = 0;
            } else {
                gas -= 1;
            }
        }

        let last_index = block.len() - 1;
        for (index, instr) in block.iter().enumerate() {
            if let Some(gas) = gas_map.get(&index) {
                self.burn_gas(*gas as i64);
                self.store_gas();
            } else if index == last_index && gas != 0 {
                self.burn_gas(gas as i64);
            }

            if let Err(e) = self.visit(instr.value, &instr.range) {
                tracing::warn!(
                    "Instruction translation failed at PC {}: {}",
                    instr.range.start,
                    e
                );
            }
        }

        // handle block termination with native CLIF control flow
        if let Some(last) = block.last() {
            if !last.value.is_termination() {
                self.burn_gas(-1);
                self.return_(Exit::ProgramNotTerminated);
            }
        }

        Ok(())
    }

    fn create_block(&mut self) -> Block {
        let block = self.builder.create_block();
        for _ in 0..14 {
            self.builder.append_block_param(block, types::I64);
        }
        block
    }
}
