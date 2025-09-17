//! Translator API V2

use crate::{Exit, Translator};
use anyhow::Result;
use cranelift::{
    codegen::ir::BlockCall,
    prelude::{InstBuilder, IntCC, JumpTableData, types},
};
use parser::{Instruction, reader::Offset};
use pvm::{MemoryInfo, Visitor};
use std::collections::BTreeMap;

const ACCUMULATE_PC: u64 = 5;
const REFINE_PC: u64 = 0;
const TEST_PC: u64 = 13;

impl Translator<'_> {
    /// Translate the main function
    ///
    /// We need to init all block parameters to our registers here.
    ///
    /// [ctx, memory, gas, [..registers]]
    pub fn translate(
        &mut self,
        registers: [u64; pvm::REGISTER_COUNT],
        func: BTreeMap<u64, Vec<Offset<Instruction>>>,
        info: MemoryInfo,
    ) -> Result<()> {
        self.memory = info;
        let entry = self.builder.create_block();
        self.builder.append_block_params_for_function_params(entry);
        self.builder.switch_to_block(entry);

        // init all registers and blocks
        let pc = self.init_pool(entry, registers);
        for pc in func.keys() {
            let block = self.builder.create_block();
            self.blocks.insert(*pc, block);
        }

        // Create jump table first before adding instructions to entry block
        self.create_jump_table()?;

        // Add the initial minimal dispatch logic in the entry block
        {
            let five = self.builder.ins().iconst(types::I64, ACCUMULATE_PC as i64);
            let thirteen = self.builder.ins().iconst(types::I64, TEST_PC as i64);
            let refine = *self.blocks.get(&REFINE_PC).unwrap_or(&self.masm.trap);
            let accumulate = *self.blocks.get(&ACCUMULATE_PC).unwrap_or(&self.masm.trap);
            let test = *self.blocks.get(&TEST_PC).unwrap_or(&self.masm.trap);
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
        self.build_macros();
        for (pc, block) in self.blocks.clone() {
            let instructions = &func[&pc];
            self.builder.switch_to_block(block);
            self.translate_block(instructions)?;
        }

        // seal all blocks
        self.builder.seal_all_blocks();
        Ok(())
    }

    /// translate a block and check termination
    ///
    /// TODO: introduce the gas map in context
    pub fn translate_block(&mut self, block: &[Offset<Instruction>]) -> Result<()> {
        /* let mut gas_map = BTreeMap::new();
        let mut gas = 0;
        for (index, instr) in block.iter().enumerate() {
            if instr.value.is_memory_op() {
                gas_map.insert(index, gas - 1);
                gas = 0;
            } else {
                gas -= 1;
            }
        } */

        // let last_index = block.len() - 1;
        for instr in block.iter() {
            /* if let Some(gas) = gas_map.get(&index) {
                self.burn_gas(*gas as i64);
                self.store_gas();
            } else if index == last_index && gas != 0 {
                self.burn_gas(gas as i64);
            } */

            self.context.burn_gas(instr)?;
            self.store_gas();
            self.visit(instr.value, &instr.range)?;
        }

        // handle block termination with native CLIF control flow
        if let Some(last) = block.last()
            && !last.value.is_termination()
        {
            self.context.burn_gas_imm(-1)?;
            self.return_(Exit::ProgramNotTerminated);
        }

        Ok(())
    }

    /// Create jump table
    pub fn create_jump_table(&mut self) -> Result<()> {
        // Generate the runtime jump table for djump instructions
        let default = BlockCall::new(
            self.masm.trap,
            std::iter::empty(),
            &mut self.builder.func.dfg.value_lists,
        );

        // Create block calls pointing directly to target blocks (no adapters needed)
        let mut calls = Vec::with_capacity(self.jump.len());
        for &jump_pc in &self.jump {
            let target = self.blocks.get(&jump_pc).copied().unwrap_or(self.masm.trap);
            let call = BlockCall::new(
                target,
                std::iter::empty(),
                &mut self.context.builder.func.dfg.value_lists,
            );
            calls.push(call);
        }

        // Create and cache the jump table
        let jt_data = JumpTableData::new(default, &calls);
        self.rt_jump_table = self.builder.create_jump_table(jt_data);
        Ok(())
    }
}
