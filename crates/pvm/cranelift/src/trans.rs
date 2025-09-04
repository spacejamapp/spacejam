//! Translator API V2

use crate::{ir, Translator};
use anyhow::Result;
use cranelift::prelude::InstBuilder;
use cranelift_codegen::ir::FuncRef;
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

    /// Translate the dispatching table
    pub fn translate_dispatcher_v2(&mut self, table: *const u8) -> Result<()> {
        let entry = self.builder.create_block();
        self.builder.append_block_params_for_function_params(entry);
        self.builder.switch_to_block(entry);

        // extract the arguments
        //
        // - input arguments: [ctx, target, a0, a1, a2, gas]
        // - output arguments: [ctx, a0, a1, a2, a3, gas]
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
}
