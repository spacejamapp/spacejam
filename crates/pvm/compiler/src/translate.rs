//! Cranelift JIT backend

use crate::{Compiler, Module};
use anyhow::Result;
use cranelift::prelude::FunctionBuilderContext;
use cranelift_codegen::ir::UserFuncName;
use cranelift_module::{FuncId, Linkage, Module as _};
use pvm::{score::OpaqueHash, Program};
use std::collections::BTreeMap;
use translator::{ir, Translator};

const MAIN: &str = "main";

impl Compiler {
    /// Compile the program with cache
    pub fn compile_with_cache(
        &mut self,
        program: &Program,
        _hash: Option<OpaqueHash>,
    ) -> Result<Module> {
        self.compile(program)
    }

    /// Declare functions for the program
    pub fn compile(&mut self, program: &Program) -> Result<Module> {
        let blob = program.blob()?;
        let format = ir::IR::from(&blob);
        let memory = crate::Memory::new(&program.memory)?;
        let minfo = program.memory.info.clone();
        tracing::debug!(
            "jump table({} = {}): {:?}",
            blob.jump_table.len(),
            format.functions.len(),
            blob.jump_table
        );

        // 1. declare all functions
        let (main, funcs) = {
            let main =
                self.module
                    .declare_function(MAIN, Linkage::Export, &format.main.signature)?;
            self.context.func.signature = format.main.signature.clone();
            let funcs = self.declare(&format)?;
            (main, funcs)
        };
        tracing::debug!("main function id: {:?}", main);

        // 2. define the main function
        let mut registers = {
            let mut translator = Translator::new(&[], &mut self.context.func, &mut self.ctx)?;
            translator.jump = blob.jump_table.clone();
            translator.translate_main(&format.main, minfo.clone())?;
            translator.pool
        };
        self.module.define_function(main, &mut self.context)?;
        println!("{}", &self.context.func.display());
        self.clear();

        // 3. define all other functions
        for (idx, (_pc, (id, func))) in funcs.iter().enumerate() {
            self.context.func.name = UserFuncName::user(1, idx as u32);
            self.context.func.signature = func.signature.clone();
            {
                let mut translator = Translator::new(
                    func.blocks.keys().copied().collect::<Vec<_>>().as_slice(),
                    &mut self.context.func,
                    &mut self.ctx,
                )?;
                translator.pool = registers;
                translator.jump = blob.jump_table.clone();
                translator.translate(func, minfo.clone())?;
                registers = translator.pool;
            }
            self.module.define_function(*id, &mut self.context)?;
            println!("{}", &self.context.func.display());
            self.clear();
        }

        // 4. finalize the compilation
        self.module.finalize_definitions()?;
        let dispatch = self.create_dispatch_table(&funcs)?;
        Ok(Module {
            code: self.module.get_finalized_function(main),
            memory,
            registers: program.registers,
            dispatch,
        })
    }

    /// create the function table
    fn create_dispatch_table<'f>(
        &self,
        table: &BTreeMap<u64, (FuncId, &'f ir::Function)>,
    ) -> Result<[u64; pvm::MAX_FUNCTIONS]> {
        let mut dispatch = [0; pvm::MAX_FUNCTIONS];
        for (idx, (id, _func)) in table.values().enumerate() {
            dispatch[idx] = self.module.get_finalized_function(*id) as u64;
        }
        Ok(dispatch)
    }

    /// Declare all functions
    fn declare<'a>(
        &mut self,
        format: &'a ir::IR,
    ) -> Result<BTreeMap<u64, (FuncId, &'a ir::Function)>> {
        let mut funcs = BTreeMap::new();
        for (pc, func) in &format.functions {
            let id = self.module.declare_function(
                format!("local_{pc}").as_str(),
                Linkage::Local,
                &func.signature,
            )?;

            tracing::debug!("declare function local_{pc}: {}", id);
            self.module.declare_func_in_func(id, &mut self.context.func);
            funcs.insert(*pc, (id, func));
        }
        Ok(funcs)
    }

    fn clear(&mut self) {
        self.context.clear();
        self.ctx = FunctionBuilderContext::new();
    }
}
