//! Object module

use crate::{
    Engine, Executable,
    module::{self, ModuleLike},
};
use anyhow::Result;
use cranelift::{
    module::default_libcall_names,
    object::{self, ObjectBuilder},
};
use pvm::{Argument, Program, Reason};
use translator::Exit;

/// Object module
pub struct ObjectModule {
    module: Option<object::ObjectModule>,
    object: Vec<u8>,
    exec: Executable,
}

impl ModuleLike for ObjectModule {
    fn new<X: Argument>() -> Result<Self> {
        let isa = Engine::compilation()?;
        let builder = ObjectBuilder::new(isa, "spacevm", default_libcall_names())?;
        let module = object::ObjectModule::new(builder);
        Ok(Self {
            module: Some(module),
            object: vec![],
            exec: Executable::default(),
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
        self.exec.load::<()>(&object)?;
        self.object = object;
        Ok(self)
    }

    fn execute<X: Argument>(&self, ctx: &mut X, pc: u64) -> Result<Reason> {
        let main = self.exec.get("main")?;
        let main_fn: super::MainSig<X> = unsafe { std::mem::transmute(main) };
        let (gas, exit_code) = main_fn(ctx, pc);
        ctx.set_gas(gas as u64);
        Ok(Exit::to_reason(exit_code))
    }
}
