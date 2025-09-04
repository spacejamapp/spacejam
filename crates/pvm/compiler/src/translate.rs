//! Cranelift JIT backend

use crate::{Module, JIT};
use anyhow::Result;
use cranelift::prelude::{types, AbiParam, FunctionBuilderContext, Signature};
use cranelift_codegen::{ir::FuncRef, isa::CallConv, Context};
use cranelift_module::{FuncId, Linkage, Module as _};
use pvm::{MemoryInfo, Program};
use std::collections::BTreeMap;
use translator::{ir, Translator};

const MAIN: &str = "main";
const DISPATCHER: &str = "dispatcher";

impl JIT {
    /// Declare functions for the program
    pub fn compile_v2(&mut self, program: &Program) -> Result<Module> {
        let blob = program.blob()?;
        let format = ir::IR::from(&blob);
        let memory = crate::Memory::new(&program.memory)?;
        let minfo = program.memory.info.clone();
        let mut context = self.module.make_context();

        // 1. create signatures
        let [main_sig, diaptcher_sig] = [
            Signature {
                params: vec![AbiParam::new(types::I64); 1],
                returns: vec![AbiParam::new(types::I64); 1],
                call_conv: CallConv::SystemV,
            },
            Signature {
                params: vec![AbiParam::new(types::I64); 15],
                returns: vec![AbiParam::new(types::I64); 14],
                call_conv: CallConv::SystemV,
            },
        ];

        // 1. declare the main function
        let main = self
            .module
            .declare_function(MAIN, Linkage::Export, &main_sig)?;
        context.func.signature = main_sig;

        // 2. declare all functions
        let (table, funcs) = self.declare(&format, &mut context)?;

        // 3. declare the dispatcher function
        let dispatcher_id =
            self.module
                .declare_function(DISPATCHER, Linkage::Local, &diaptcher_sig)?;
        let dispatcher = self
            .module
            .declare_func_in_func(dispatcher_id, &mut context.func);

        // 4. define the main function
        self.translate_main(&mut context, dispatcher, minfo.clone())?;
        self.module.define_function(main, &mut context)?;
        context.clear();

        // 5. define the dispatcher function
        context.func.signature = diaptcher_sig;
        self.translate_dispatcher(&mut context, &table)?;
        self.module.define_function(dispatcher_id, &mut context)?;
        context.clear();

        // 6. define all other functions
        for (id, func) in funcs {
            context.func.signature = func.signature.clone();
            self.translate_func(&mut context, dispatcher, func, &table, minfo.clone())?;
            self.module.define_function(id, &mut context)?;
            context.clear();
        }

        // 7. finalize the compilation
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
        dispatcher: FuncRef,
        func: &ir::Function,
        table: &BTreeMap<u64, FuncRef>,
        info: MemoryInfo,
    ) -> Result<()> {
        let mut bctx = FunctionBuilderContext::new();
        context.func.signature = func.signature.clone();

        // translate the function
        let mut trans = Translator::new(&mut context.func, &mut bctx)?;
        trans.funcs = table.clone();
        trans.translate_v2(dispatcher, func, info)?;
        Ok(())
    }

    /// Translate the main function
    pub fn translate_dispatcher(
        &mut self,
        context: &mut Context,
        table: &BTreeMap<u64, FuncRef>,
    ) -> Result<()> {
        let mut bctx = FunctionBuilderContext::new();
        let mut trans = Translator::new(&mut context.func, &mut bctx)?;
        trans.translate_dispatcher_v2(table)?;
        Ok(())
    }

    /// Translate the main function
    pub fn translate_main(
        &mut self,
        ctx: &mut Context,
        dispatcher: FuncRef,
        info: MemoryInfo,
    ) -> Result<()> {
        let mut bctx = FunctionBuilderContext::new();
        let mut trans = Translator::new(&mut ctx.func, &mut bctx)?;
        trans.translate_main(info, dispatcher)?;
        Ok(())
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
