//! Object module

use crate::{
    Artifact, Engine, Executable, host,
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
    exec: Executable,
}

impl ModuleLike for ObjectModule {
    fn new<X: Argument>() -> Result<Self> {
        let isa = Engine::compilation()?;
        let builder = ObjectBuilder::new(isa, "spacevm", default_libcall_names())?;
        let module = object::ObjectModule::new(builder);
        Ok(Self {
            module: Some(module),
            exec: Executable::default(),
        })
    }

    fn compile(mut self, program: &Program) -> Result<Self> {
        let info = program.meta.info();
        let name = format!(
            "{}-{}-{}.o",
            info.name,
            info.version,
            &hex::encode(crypto::blake3(program.code.as_ref()))[..6]
        );
        if let Some(object) = Artifact::get("lib", &name) {
            self.exec.load::<()>(&object)?;
            return Ok(self);
        }

        let Some(mut module) = self.module.take() else {
            return Err(anyhow::anyhow!("module not found"));
        };

        module::compile(&mut module, program)?;
        let object = module.finish().emit()?;
        Artifact::set("lib", &name, &object)?;
        self.exec.load::<()>(&object)?;
        Ok(self)
    }

    fn execute<X: Argument>(
        &self,
        ctx: &mut pvm::Context<'_, X, crate::Memory>,
        pc: u64,
    ) -> Result<Reason> {
        let main = self.exec.get("main")?;
        let main_fn: super::MainSig<X> = unsafe { std::mem::transmute(main) };
        let (gas, exit_code) = main_fn(ctx, pc, host::table::<X>());
        ctx.set_gas(gas as u64);
        Ok(Exit::to_reason(exit_code))
    }
}

unsafe impl Send for ObjectModule {}
unsafe impl Sync for ObjectModule {}
