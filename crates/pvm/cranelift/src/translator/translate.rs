//! translation utils

use crate::{constants::PVM_REGISTER_COUNT, translator::Block, Translator};
use anyhow::Result;
use cranelift::prelude::*;
use cranelift_codegen::ir;
use parser::Visitor;
use std::collections::HashMap;

impl Translator<'_> {
    /// Translate entire program
    pub fn translate(&mut self, entry: ir::Block, blocks: &HashMap<u64, Block>) -> Result<()> {
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
        for (pc, pvm_block) in blocks {
            let cranelift_block = self.blocks[&pc];
            self.builder.switch_to_block(cranelift_block);

            // Translate instructions in this block using shared translator
            match self.translate_block(&pvm_block) {
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
}
