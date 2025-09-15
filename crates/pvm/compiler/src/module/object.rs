//! Object module

use crate::{
    Engine,
    module::{self, ModuleLike},
};
use anyhow::Result;
use cranelift::{
    module::default_libcall_names,
    object::{self, ObjectBuilder},
};
use pvm::{Argument, Program, Reason};

/// Object module
pub struct ObjectModule {
    module: Option<object::ObjectModule>,
    object: Vec<u8>,
}

impl ModuleLike for ObjectModule {
    fn new<X: Argument>() -> Result<Self> {
        let isa = Engine::compilation()?;
        let builder = ObjectBuilder::new(isa, "spacevm", default_libcall_names())?;
        let module = object::ObjectModule::new(builder);
        Ok(Self {
            module: Some(module),
            object: vec![],
        })
    }

    fn compile(mut self, program: &Program) -> Result<Self> {
        let Some(mut module) = self.module.take() else {
            return Err(anyhow::anyhow!("module not found"));
        };

        let artifact = module::compile(&mut module, program)?;
        let object = module.finish().emit()?;
        let info = program.meta.info();
        let name = format!("{}-{}.o", info.name, info.version);
        artifact.save(&name, &object)?;
        self.object = object;
        Ok(self)
    }

    fn execute<X: Argument>(&self, _ctx: &mut X, _pc: u64) -> Result<Reason> {
        todo!()
    }
}
