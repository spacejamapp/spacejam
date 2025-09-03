//! Translator API V2

use crate::{ir, Translator};
use anyhow::Result;
use cranelift_codegen::ir::{StackSlot, StackSlotData, StackSlotKind};

impl Translator<'_> {
    /// Translate a regular PVM function (non-main)
    pub fn translate_v2(&mut self, fun: ir::Function, stack: StackSlot) -> Result<()> {
        self.pool.stack = stack;
        let entry = self.builder.create_block();
        self.builder.append_block_params_for_function_params(entry);
        self.builder.switch_to_block(entry);
        self.stack_load_params();
        for block in &fun.blocks {
            self.translate_block(block)?;
        }

        self.stack_store_params();
        self.builder.seal_all_blocks();
        Ok(())
    }

    /// Translate the dispatcher/main function
    /// Creates the shared stack and handles initial setup
    pub fn translate_dispatcher_v2(&mut self, main: ir::Function) -> Result<StackSlot> {
        let entry = self.builder.create_block();
        self.builder.append_block_params_for_function_params(entry);
        self.builder.switch_to_block(entry);

        // Create shared stack slot for 14 values (13 registers + gas)
        let stack = self.builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            // 14 values * 8 bytes each
            14 * 8,
            // 8-byte alignment (2^3 = 8)
            3,
        ));

        // TODO: Initialize context (adapt from existing init_context)
        //
        // - init with the current ir::Function
        let _ctx_ptr = self.builder.block_params(entry)[0];
        // self.init_context_v2(ctx_ptr);
        self.load_params();

        // TODO: Implement entry point logic (adapted from translate_entry)
        // - Check PC for accumulate/test/general
        // - Use function calls instead of br_table
        // self.translate_entry_v2(entry, shared_stack)?;

        // Translate main function blocks
        for block in &main.blocks {
            self.translate_block(block)?;
        }

        // Store final state to shared stack
        self.stack_store_params();
        self.builder.seal_all_blocks();
        Ok(stack)
    }

    // TODO: Helper methods to implement
    // fn init_context_v2(&mut self, ctx_ptr: Value) -> Result<()>
    // fn translate_entry_v2(&mut self, entry: Block, shared_stack: StackSlot) -> Result<()>
    // fn translate_block_v2(&mut self, block: &Vec<Offset<Instruction>>) -> Result<()>
}
