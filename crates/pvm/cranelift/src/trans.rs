//! Translator API V2

use std::collections::BTreeMap;

use crate::{ir, Exit, Translator};
use anyhow::Result;
use cranelift::prelude::{types, InstBuilder, JumpTableData};
use cranelift_codegen::ir::{Block, BlockArg, BlockCall, FuncRef};
use pvm::MemoryInfo;

impl Translator<'_> {
    /// Translate a regular PVM function (non-main)
    pub fn translate_v2(
        &mut self,
        dispatcher: FuncRef,
        fun: &ir::Function,
        info: MemoryInfo,
    ) -> Result<()> {
        self.dispatcher = dispatcher;
        let entry = self.builder.create_block();
        self.builder.append_block_params_for_function_params(entry);
        self.builder.switch_to_block(entry);
        self.init_context(self.builder.block_params(entry)[0], info);
        for block in &fun.blocks {
            self.translate_block(block)?;
        }

        self.builder.seal_all_blocks();
        Ok(())
    }

    /// Translate the main function
    pub fn translate_main(&mut self, info: MemoryInfo, dispatcher: FuncRef) -> Result<()> {
        let entry = self.builder.create_block();
        self.builder.append_block_params_for_function_params(entry);
        self.builder.switch_to_block(entry);
        self.init_context(self.builder.block_params(entry)[0], info);
        self.load_params();
        let args = self.func_args();
        self.builder.ins().return_call(dispatcher, &args);
        Ok(())
    }

    /// Translate the dispatcher/main function
    /// Creates the shared stack and handles initial setup
    pub fn translate_dispatcher_v2(&mut self, table: &BTreeMap<u64, FuncRef>) -> Result<()> {
        self.funcs = table.clone();
        let entry = self.builder.create_block();
        self.builder.append_block_params_for_function_params(entry);
        self.builder.switch_to_block(entry);
        let func_args = self.builder.block_params(entry)[1..].to_vec();
        let block_args = func_args
            .iter()
            .map(|arg| BlockArg::Value(*arg))
            .collect::<Vec<_>>();

        // Generate the trap block
        let trap = self.builder.create_block();
        self.builder.switch_to_block(trap);
        self.return_(Exit::InvalidJumpTarget);

        // Generate the default block
        let default = BlockCall::new(
            trap,
            std::iter::empty(),
            &mut self.builder.func.dfg.value_lists,
        );

        // Create block calls pointing to functions
        let mut block_calls = Vec::with_capacity(self.jump.len());
        for (_, funcref) in self.funcs.clone() {
            let call = self.create_block();
            self.builder.switch_to_block(call);
            self.builder.ins().return_call(funcref, &func_args);
            block_calls.push(BlockCall::new(
                call,
                block_args.clone(),
                &mut self.builder.func.dfg.value_lists,
            ));
        }

        // Create and cache the jump table
        let jt_data = JumpTableData::new(default, &block_calls);
        self.rt_jump_table = self.builder.create_jump_table(jt_data);
        let target = self.builder.block_params(entry)[0];
        self.builder.ins().br_table(target, self.rt_jump_table);
        self.builder.seal_all_blocks();
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
}
