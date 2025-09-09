//! Translator API V2

use crate::{ir, offsets, Exit, Translator};
use anyhow::Result;
use cranelift::prelude::{types, InstBuilder, IntCC, JumpTableData, MemFlags};
use cranelift_codegen::ir::BlockCall;
use parser::{reader::Offset, Instruction};
use pvm::{MemoryInfo, Visitor};
use std::collections::BTreeMap;

const ACCUMULATE_PC: u64 = 5;
const REFINE_PC: u64 = 0;
const TEST_PC: u64 = 13;

impl Translator<'_> {
    /// Translate a regular PVM function (non-main)
    pub fn translate(&mut self, fun: &ir::Function, _info: MemoryInfo) -> Result<()> {
        // create all blocks
        for (idx, pc) in fun.blocks.keys().enumerate() {
            let block = if idx == 0 {
                let block = self.builder.create_block();
                self.builder.append_block_params_for_function_params(block);
                block
            } else {
                self.builder.create_block()
            };
            self.blocks.insert(*pc, block);
        }

        // translate the all blocks
        for (pc, block) in self.blocks.clone() {
            let instructions = &fun.blocks[&pc];
            self.builder.switch_to_block(block);
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
        let params = self.builder.block_params(entry).to_vec();
        let [vmctx, pc, gas] = [params[0], params[1], params[2]];
        self.pool.vmctx = vmctx;
        self.pool.memory = self.builder.ins().load(
            types::I64,
            MemFlags::trusted(),
            vmctx,
            offsets::MEMORY_OFFSET,
        );

        // init all variables
        self.pool.gas = self.builder.declare_var(types::I64);
        self.builder.def_var(self.pool.gas, gas);
        for i in 0..13 {
            let reg = self.builder.declare_var(types::I64);
            self.pool.registers[i] = reg;
            self.builder.def_var(reg, params[i + 3]);
        }

        // create all blocks
        for pc in func.blocks.keys() {
            let block = self.builder.create_block();
            self.blocks.insert(*pc, block);
        }

        // Create jump table first before adding instructions to entry block
        self.create_jump_table()?;

        // Add the initial minimal dispatch logic in the entry block
        {
            let five = self.builder.ins().iconst(types::I64, ACCUMULATE_PC as i64);
            let thirteen = self.builder.ins().iconst(types::I64, TEST_PC as i64);
            let refine = self.blocks[&REFINE_PC];
            let accumulate = self.blocks.get(&ACCUMULATE_PC).cloned().unwrap_or(refine);
            let test = self.blocks.get(&TEST_PC).cloned().unwrap_or(refine);
            let check_test = self.builder.create_block();

            // build the initial condition in the entry block
            let is_accumulate = self.builder.ins().icmp(IntCC::Equal, pc, five);
            self.builder
                .ins()
                .brif(is_accumulate, accumulate, &[], check_test, &[]);
            self.builder.seal_block(entry);

            // build the check test block
            self.builder.switch_to_block(check_test);
            let is_test = self.builder.ins().icmp(IntCC::Equal, pc, thirteen);
            self.builder.ins().brif(is_test, test, &[], refine, &[]);
            self.builder.seal_block(check_test);
        }

        // now translate the rest of the blocks
        for (pc, block) in self.blocks.clone() {
            let instructions = &func.blocks[&pc];
            self.builder.switch_to_block(block);
            self.translate_block(instructions)?;
        }

        // Fill the trap block if it was created
        self.builder.switch_to_block(self.trap);
        self.return_(Exit::InvalidJumpTarget);
        self.builder.seal_block(self.trap);
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

    /// Create jump table
    pub fn create_jump_table(&mut self) -> Result<()> {
        // Generate the runtime jump table for djump instructions
        let default = BlockCall::new(
            self.trap,
            std::iter::empty(),
            &mut self.builder.func.dfg.value_lists,
        );

        // Create block calls pointing directly to target blocks (no adapters needed)
        let mut calls = Vec::with_capacity(self.jump.len());
        for &jump_pc in &self.jump {
            let target = self.blocks.get(&jump_pc).copied().unwrap_or(self.trap);
            let call = BlockCall::new(
                target,
                std::iter::empty(),
                &mut self.builder.func.dfg.value_lists,
            );
            calls.push(call);
        }

        // Create and cache the jump table
        let jt_data = JumpTableData::new(default, &calls);
        self.rt_jump_table = self.builder.create_jump_table(jt_data);
        Ok(())
    }
}
