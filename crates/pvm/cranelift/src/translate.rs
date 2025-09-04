//! Translator API V2

use crate::{ir, Exit, Translator};
use anyhow::Result;
use parser::{reader::Offset, Instruction};
use pvm::{MemoryInfo, Visitor};
use std::collections::BTreeMap;

impl Translator<'_> {
    /// Translate a regular PVM function (non-main)
    pub fn translate(&mut self, fun: &ir::Function, _info: MemoryInfo) -> Result<()> {
        let entry = self.builder.create_block();
        self.builder.append_block_params_for_function_params(entry);
        self.builder.switch_to_block(entry);
        for block in fun.blocks.values() {
            self.translate_block(block)?;
        }

        self.builder.seal_all_blocks();
        Ok(())
    }

    /// Translate the main function
    ///
    /// We need to init all block parameters to our registers here.
    pub fn translate_main(&mut self, _info: MemoryInfo) -> Result<()> {
        let entry = self.builder.create_block();
        self.builder.append_block_params_for_function_params(entry);
        self.builder.switch_to_block(entry);
        // self.builder.ins().return_call(dispatcher, &[]);
        Ok(())
    }

    /// translate a block and check termination
    pub fn translate_block(&mut self, block: &[Offset<Instruction>]) -> Result<()> {
        let mut gas_map = BTreeMap::new();
        let mut gas = 0;
        for (index, instr) in block.iter().enumerate() {
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
                self.sync_gas();
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
}
