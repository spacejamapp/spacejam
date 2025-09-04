//! Cranelift JIT backend

use crate::{Module, JIT};
use anyhow::Result;
use cranelift::prelude::FunctionBuilderContext;
use cranelift_codegen::{
    ir::{FuncRef, StackSlot},
    Context,
};
use cranelift_module::{FuncId, Linkage, Module as _};
use pvm::{MemoryInfo, Program};
use std::collections::BTreeMap;
use translator::{ir, Translator};

const MAIN: &str = "main";

impl JIT {
    /// Declare functions for the program
    pub fn compile_v2(&mut self, program: &Program) -> Result<Module> {
        let blob = program.blob()?;
        let format = ir::IR::from(&blob);
        let memory = crate::Memory::new(&program.memory)?;
        let minfo = program.memory.info.clone();

        // 1. declare all functions
        let main = self
            .module
            .declare_function(MAIN, Linkage::Export, &format.main.signature)?;

        // 1. translate the main function
        let (mut context, stack) = self.translate_main(&format.main, minfo.clone())?;
        self.module.define_function(main, &mut context)?;
        context.clear();

        // 2. translate the rest of the functions
        let (table, funcs) = self.declare(&format, &mut context)?;
        for (id, func) in funcs {
            self.translate_func(&mut context, func, stack, &table, minfo.clone())?;
            self.module.define_function(id, &mut context)?;
        }

        // 3. finalize the compilation
        self.module.finalize_definitions()?;
        Ok(Module {
            code: self.module.get_finalized_function(main),
            memory,
            registers: program.registers,
        })
    }

    /// Translate a single function
    pub fn translate_func(
        &mut self,
        context: &mut Context,
        func: &ir::Function,
        stack: StackSlot,
        table: &BTreeMap<u64, FuncRef>,
        info: MemoryInfo,
    ) -> Result<()> {
        let mut bctx = FunctionBuilderContext::new();
        context.func.signature = func.signature.clone();

        // translate the function
        let mut trans = Translator::new(&mut context.func, &mut bctx)?;
        trans.funcs = table.clone();
        trans.translate_v2(func, stack, info)?;
        Ok(())
    }

    /// Translate the main function
    pub fn translate_main(
        &mut self,
        main: &ir::Function,
        info: MemoryInfo,
    ) -> Result<(Context, StackSlot)> {
        let mut ctx = self.module.make_context();
        let mut bctx = FunctionBuilderContext::new();
        let mut trans = Translator::new(&mut ctx.func, &mut bctx)?;
        let stack = trans.translate_dispatcher_v2(main, info)?;
        Ok((ctx, stack))
    }

    /// Declare all functions
    pub fn declare<'a>(
        &mut self,
        format: &'a ir::IR,
        context: &mut Context,
    ) -> Result<(BTreeMap<u64, FuncRef>, BTreeMap<FuncId, &'a ir::Function>)> {
        let mut table = BTreeMap::new();
        let mut funcs = BTreeMap::new();
        for (pc, func) in &format.functions {
            let id = self.module.declare_function(
                format!("{pc}").as_str(),
                Linkage::Local,
                &func.signature,
            )?;

            let funref = self.module.declare_func_in_func(id, &mut context.func);
            funcs.insert(id, func);
            table.insert(*pc, funref);
        }
        Ok((table, funcs))
    }
}
