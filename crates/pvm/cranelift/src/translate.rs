//! Translator API V2

use crate::{ir, Exit, Translator};
use anyhow::Result;
use cranelift::prelude::InstBuilder;
use cranelift_codegen::ir::FuncRef;
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
    pub fn translate_main(&mut self, _info: MemoryInfo, dispatcher: FuncRef) -> Result<()> {
        let entry = self.builder.create_block();
        self.builder.append_block_params_for_function_params(entry);
        self.builder.switch_to_block(entry);
        self.builder.ins().return_call(dispatcher, &[]);
        Ok(())
    }

    /// Translate the dispatching table
    pub fn translate_dispatcher(&mut self, table: *const u8) -> Result<()> {
        let entry = self.builder.create_block();
        self.builder.append_block_params_for_function_params(entry);
        self.builder.switch_to_block(entry);

        // extract the arguments
        //
        // - input arguments: [ctx, target, a0, a1, a2, gas]
        // - output arguments: [ctx, a0, a1, a2, a3, a4, gas]
        //
        // TODO: load values from stack to balance the arguments
        let target = self.builder.block_params(entry)[0];
        let func_args = self.builder.block_params(entry)[1..].to_vec();
        let sig = self.builder.import_signature(crate::ir::sig());

        // call the table
        let table = self.builder.ins().iadd_imm(target, table as i64);
        self.builder.ins().call_indirect(sig, table, &func_args);
        self.builder.seal_all_blocks();
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
