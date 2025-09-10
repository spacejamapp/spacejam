//! Cranelift JIT backend

use crate::{Compiler, Module};
use anyhow::Result;
use cranelift::prelude::{types, AbiParam, Signature};
use cranelift_codegen::{control::ControlPlane, ir::Function, isa::CallConv};
use cranelift_module::{Linkage, Module as _, ModuleReloc};
use pvm::{score::OpaqueHash, Program};
use translator::Translator;

const MAIN: &str = "main";

impl Compiler {
    /// Compile the program with cache
    pub fn compile_with_cache(
        &mut self,
        program: &Program,
        hash: Option<OpaqueHash>,
    ) -> Result<Module> {
        self.compile(program, hash)
    }

    /// Declare functions for the program
    pub fn compile(&mut self, program: &Program, _hash: Option<OpaqueHash>) -> Result<Module> {
        let memory = crate::Memory::new(&program.memory)?;
        let signature = Signature {
            params: vec![AbiParam::new(types::I64); 16],
            returns: vec![AbiParam::new(types::I64); 2],
            call_conv: CallConv::Fast,
        };
        let main = {
            let main = self
                .module
                .declare_function(MAIN, Linkage::Export, &signature)?;
            self.context.func.signature = signature.clone();
            main
        };

        // compile the program with cache
        let func = self.translate(program)?;
        let isa = self.module.isa();
        let mut cpanel = ControlPlane::default();
        let (compiled, _hits) = self
            .context
            .compile_with_cache(isa, &mut self.artifact, &mut cpanel)
            .map_err(|e| anyhow::anyhow!("failed to compile program: {:?}", e))?;

        // relocate the function
        let relocs = compiled
            .buffer
            .relocs()
            .iter()
            .map(|r| ModuleReloc::from_mach_reloc(r, &func, main))
            .collect::<Vec<_>>();

        self.module
            .define_function_bytes(main, 1, compiled.code_buffer(), &relocs)?;
        self.module.finalize_definitions()?;
        compiled.buffer.data();

        Ok(Module {
            object: Default::default(),
            fun: self.module.get_finalized_function(main),
            memory,
            registers: program.registers,
        })
    }

    /// Translate the program to CLIF
    fn translate(&mut self, program: &Program) -> Result<Function> {
        let host = self.declare_host_in_module()?;
        let blob = program.blob()?;
        let code = blob.read_blocks()?;
        let host = self.declare_host_in_func(host)?;
        let minfo = program.memory.info.clone();
        let mut translator = Translator::new(&[], &mut self.context.func, &mut self.ctx)?;
        translator.jump = blob.jump_table.clone();
        translator.host = host;
        translator.translate(code, minfo.clone())?;
        Ok(self.context.func.clone())
    }
}
