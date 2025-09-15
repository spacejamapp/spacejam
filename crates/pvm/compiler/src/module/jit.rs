//! JIT module

use crate::{
    Engine,
    module::{self, MainSig, ModuleLike},
};
use anyhow::Result;
use cranelift::{
    jit,
    jit::JITBuilder,
    module::{FuncId, default_libcall_names},
};
use pvm::{Argument, Program};

/// JIT module
pub struct JITModule(jit::JITModule);

impl ModuleLike for JITModule {
    fn new<X: Argument>() -> Result<Self> {
        let isa = Engine::compilation()?;
        let builder = JITBuilder::with_isa(isa, default_libcall_names());
        let module = jit::JITModule::new(builder);
        Ok(Self(module))
    }

    fn compile(mut self, program: &Program) -> Result<Self> {
        module::compile(&mut self.0, program)?;
        self.0.finalize_definitions()?;
        Ok(self)
    }

    fn main<X: Argument>(&self) -> Result<MainSig<X>> {
        let main = FuncId::from_u32(0);
        let func = unsafe {
            std::mem::transmute::<*const u8, module::MainSig<X>>(
                self.0.get_finalized_function(main),
            )
        };
        Ok(func)
    }
}

unsafe impl Send for JITModule {}
unsafe impl Sync for JITModule {}
