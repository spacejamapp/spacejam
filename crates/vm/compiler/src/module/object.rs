//! Object module

use crate::{
    Artifact, Engine, Executable,
    module::{self, MainSig, ModuleLike},
};
use anyhow::Result;
use cranelift::{
    module::default_libcall_names,
    object::{self, ObjectBuilder},
};
use pvm::{Argument, Program};

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

    fn main<X: Argument>(&self) -> Result<MainSig<X>> {
        let main = self.exec.get("main")?;
        Ok(unsafe { std::mem::transmute::<usize, MainSig<X>>(main) })
    }
}

unsafe impl Send for ObjectModule {}
unsafe impl Sync for ObjectModule {}
