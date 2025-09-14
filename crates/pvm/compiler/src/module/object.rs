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
    object: Option<object::ObjectProduct>,
}

impl ModuleLike for ObjectModule {
    fn new<X: Argument>() -> Result<Self> {
        let isa = Engine::compilation()?;
        let builder = ObjectBuilder::new(isa, "spacevm", default_libcall_names())?;
        let module = object::ObjectModule::new(builder);
        Ok(Self {
            module: Some(module),
            object: None,
        })
    }

    fn compile(mut self, program: &Program) -> Result<Self> {
        let Some(mut module) = self.module.take() else {
            return Err(anyhow::anyhow!("module not found"));
        };

        // compile the program
        module::compile(&mut module, program)?;
        let object = module.finish();
        self.object = Some(object);

        // TODO: cache the object
        let info = program.meta.info();
        let _name = format!("{}-{}.o", info.name, info.version);

        Ok(self)
    }

    fn execute<X: Argument>(&self, ctx: &mut X, pc: u64) -> Result<Reason> {
        todo!()
    }
}
