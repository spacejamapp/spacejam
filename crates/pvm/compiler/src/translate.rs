//! Cranelift JIT backend

use crate::{Compiler, Module};
use anyhow::Result;
use cranelift::prelude::{
    types, AbiParam, FunctionBuilderContext, Signature, StackSlotData, StackSlotKind,
};
use cranelift_codegen::isa::CallConv;
use cranelift_module::{Linkage, Module as _};
use pvm::{score::OpaqueHash, Program};
use translator::Translator;

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
        let code = blob.read_blocks()?;
        let memory = crate::Memory::new(&program.memory)?;
        let minfo = program.memory.info.clone();
        let signature = Signature {
            params: vec![AbiParam::new(types::I64); 16],
            returns: vec![AbiParam::new(types::I64); 2],
            call_conv: CallConv::Fast,
        };

        // 1. declare all dynamic functions
        let main = {
            let main = self
                .module
                .declare_function(MAIN, Linkage::Export, &signature)?;
            self.context.func.signature = signature.clone();
            main
        };

        // 2. declare all host functions
        let host = self.declare_host_in_module()?;
        let host = self.declare_host_in_func(host)?;

        // 2. define the main function
        {
            let mut translator = Translator::new(&[], &mut self.context.func, &mut self.ctx)?;
            translator.jump = blob.jump_table.clone();
            translator.stack = translator
                .builder
                .create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 16, 0));
            translator.host = host;
            translator.translate(code, minfo.clone())?;
        };
        self.module.define_function(main, &mut self.context)?;
        if std::env::var("SHOW_CLIF").is_ok() {
            println!("{}", &self.context.func.display());
        }
        self.clear();

        // 4. finalize the compilation
        self.module.finalize_definitions()?;
        Ok(Module {
            code: self.module.get_finalized_function(main),
            memory,
            registers: program.registers,
        })
    }

    fn clear(&mut self) {
        self.context.clear();
        self.ctx = FunctionBuilderContext::new();
    }
}
