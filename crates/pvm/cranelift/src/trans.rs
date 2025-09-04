//! Translator API V2

use crate::{ir, Exit, Translator};
use anyhow::Result;
use cranelift::prelude::{types, InstBuilder, IntCC, Value};
use cranelift_codegen::ir::{Block, BlockArg, StackSlot, StackSlotData, StackSlotKind};
use pvm::MemoryInfo;

impl Translator<'_> {
    /// Translate a regular PVM function (non-main)
    pub fn translate_v2(
        &mut self,
        fun: &ir::Function,
        stack: StackSlot,
        info: MemoryInfo,
    ) -> Result<()> {
        self.pool.stack = stack;
        let entry = self.builder.create_block();
        self.builder.append_block_params_for_function_params(entry);
        self.builder.switch_to_block(entry);
        self.init_context(self.builder.block_params(entry)[0], info);
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
    pub fn translate_dispatcher_v2(
        &mut self,
        main: &ir::Function,
        info: MemoryInfo,
    ) -> Result<StackSlot> {
        let entry = self.builder.create_block();
        self.builder.append_block_params_for_function_params(entry);
        self.builder.switch_to_block(entry);
        self.init_context(self.builder.block_params(entry)[0], info);
        self.load_params();

        // Store final state to shared stack
        self.pool.stack = self.builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            14 * 8,
            3,
        ));

        // Translate main function blocks
        self.translate_entry_v2(entry)?;
        for block in &main.blocks {
            self.translate_block(block)?;
        }

        // Store final state to shared stack
        self.stack_store_params();
        self.builder.seal_all_blocks();
        Ok(self.pool.stack)
    }

    /// Handle the entry point
    ///
    /// - 5 for accumulate programs
    /// - 13 for testing
    /// - 0 for general programs
    fn translate_entry_v2(&mut self, entry: Block) -> Result<()> {
        self.builder.switch_to_block(entry);
        let pc = self.pc();
        let trap = self.builder.create_block();
        let [accumulate, test] = [self.create_block(); 2];

        // Check for pc == 5 (accumulate)
        let five = self.builder.ins().iconst(types::I64, 5);
        let is_accumulate = self.builder.ins().icmp(IntCC::Equal, pc, five);
        let accumulate = self.call_or(5, accumulate, trap);

        // Check for pc == 13 (testing)
        let thirteen = self.builder.ins().iconst(types::I64, 13);
        let is_test = self.builder.ins().icmp(IntCC::Equal, pc, thirteen);
        let test = self.call_or(13, test, trap);

        // Default to block 0 (general)
        let general = self.blocks.get(&0).copied().unwrap_or(trap);

        // construct the arguments for the blocks (registers + gas)
        let args = self.args();
        let empty_args: Vec<BlockArg> = vec![];
        let [accumulate_args, test_args, general_args] = [accumulate, test, general].map(|b| {
            if b == trap || self.testing {
                &empty_args[..]
            } else {
                &args[..]
            }
        });

        // Branch: if pc == 5 goto accumulate, else check for pc == 13
        let check_test = self.create_block();
        self.builder.ins().brif(
            is_accumulate,
            accumulate,
            accumulate_args,
            check_test,
            &args,
        );

        // Branch: if pc == 13 goto test, else goto general
        {
            self.builder.switch_to_block(check_test);
            let check_test_params = self.load_block_args(check_test);
            self.params(&check_test_params);
            self.builder
                .ins()
                .brif(is_test, test, test_args, general, general_args);
            self.builder.seal_block(check_test);
        }
        self.builder.seal_block(entry);

        // Trap block: invalid jump target
        self.builder.switch_to_block(trap);
        self.return_(Exit::InvalidJumpTarget);
        self.builder.seal_block(trap);
        Ok(())
    }

    /// Create a new block
    fn create_block(&mut self) -> Block {
        let block = self.builder.create_block();
        for _ in 0..14 {
            self.builder.append_block_param(block, types::I64);
        }
        block
    }

    /// Load block arguments
    fn load_block_args(&self, block: Block) -> Vec<Value> {
        (0..14)
            .map(|i| self.builder.block_params(block)[i])
            .collect::<Vec<_>>()
    }

    /// Call a function or return a default block
    fn call_or(&mut self, pc: u64, block: Block, or: Block) -> Block {
        let Some(funcref) = self.funcs.get(&pc) else {
            return or;
        };
        self.builder.switch_to_block(block);
        let block_args = self.load_block_args(block);
        self.builder.ins().return_call(*funcref, &block_args);
        block
    }
}
