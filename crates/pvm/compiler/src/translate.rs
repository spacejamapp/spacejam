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
        //
        // - main function: [ctx, a0, a1, a2, a3, gas] -> [exit]
        // - dispatcher function: [ctx, target, a0, a1, a2, gas] -> [ctx, a0, a1, a2, a3, gas]
        let [main_sig, diaptcher_sig] = [
            Signature {
                params: vec![AbiParam::new(types::I64); 6],
                returns: vec![AbiParam::new(types::I64); 1],
                call_conv: CallConv::SystemV,
            },
            Signature {
                params: vec![AbiParam::new(types::I64); 6],
                returns: vec![AbiParam::new(types::I64); 6],
                call_conv: CallConv::SystemV,
            },
        ];

        // 1. declare the main function
        let main = self
            .module
            .declare_function(MAIN, Linkage::Export, &main_sig)?;
        context.func.signature = main_sig;

        // 2. declare all functions
        let funcs = self.declare(&format, &mut context)?;

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

        // 5. define all other functions
        for (id, func) in &funcs {
            context.func.signature = func.signature.clone();
            self.translate_func(&mut context, dispatcher, func, minfo.clone())?;
            self.module.define_function(*id, &mut context)?;
            context.clear();
        }

        // 6. define the dispatcher function
        context.func.signature = diaptcher_sig;
        self.translate_dispatcher(&mut context)?;
        self.module.define_function(dispatcher_id, &mut context)?;
        context.clear();

        // 7. finalize the compilation
        self.module.finalize_definitions()?;

        // 8. create the function table
        let table = self.create_fun_table(&funcs)?;
        Ok(Module {
            code: self.module.get_finalized_function(main),
            memory,
            registers: program.registers,
            table,
        })
    }

    /// Translate a single function
    fn translate_func(
        &mut self,
        context: &mut Context,
        dispatcher: FuncRef,
        func: &ir::Function,
        info: MemoryInfo,
    ) -> Result<()> {
        let mut bctx = FunctionBuilderContext::new();
        context.func.signature = func.signature.clone();
        let mut trans = Translator::new(&mut context.func, &mut bctx)?;
        trans.translate_v2(dispatcher, func, info)?;
        Ok(())
    }

    /// create the function table
    fn create_fun_table<'f>(
        &self,
        table: &BTreeMap<FuncId, &'f ir::Function>,
    ) -> Result<*const u8> {
        let mut fun = Vec::new();
        for func in table.keys() {
            fun.push(self.module.get_finalized_function(*func));
        }
        Ok(fun.as_ptr() as *const u8)
    }

    /// Translate the main function
    fn translate_dispatcher(&mut self, context: &mut Context) -> Result<()> {
        let mut bctx = FunctionBuilderContext::new();
        let mut trans = Translator::new(&mut context.func, &mut bctx)?;
        trans.translate_dispatcher_v2(0 as *const u8)?;
        Ok(())
    }

    /// Translate the main function
    fn translate_main(
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
    fn declare<'a>(
        &mut self,
        format: &'a ir::IR,
        context: &mut Context,
    ) -> Result<BTreeMap<FuncId, &'a ir::Function>> {
        let mut funcs = BTreeMap::new();
        for (pc, func) in &format.functions {
            let id = self.module.declare_function(
                format!("local_{pc}").as_str(),
                Linkage::Local,
                &func.signature,
            )?;

            self.module.declare_func_in_func(id, &mut context.func);
            funcs.insert(id, func);
        }
        Ok(funcs)
    }
}
