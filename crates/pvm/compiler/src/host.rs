//! Host call trampoline

use crate::JIT;
use anyhow::Result;
use cranelift::prelude::{types, AbiParam};
use cranelift_codegen::ir::FuncRef;
use cranelift_module::{Linkage, Module};
use pvm::Argument;

pub const CALL: &str = "call";
pub const SBRK: &str = "sbrk";

/// Host function trampoline
///
/// This function is called from JIT-compiled code to invoke host functions.
/// It casts the raw context pointer and delegates to [pvm::host::call]
pub extern "C" fn call<X: Argument>(index: u32, ctx: *mut u8) -> u8 {
    let context = unsafe { &mut *(ctx as *mut pvm::Context<X, crate::Memory>) };
    match pvm::host::call(index, context) {
        pvm::Reason::Halt => 0,
        pvm::Reason::Panic(_) => 1,
        pvm::Reason::Fault { .. } => 2,
        pvm::Reason::HostCall(_) => 3,
        pvm::Reason::OOG => 4,
        pvm::Reason::Continue => 5,
    }
}

/// Host function trampoline
///
/// This function is called from JIT-compiled code to invoke host functions.
/// It casts the raw context pointer and delegates to [pvm::host::call]
pub extern "C" fn sbrk<X: Argument>(ctx: *mut u8, target: u8, increment: u8) {
    let context = unsafe { &mut *(ctx as *mut pvm::Context<X, crate::Memory>) };
    context.sbrk(target, increment);
}

impl JIT {
    /// Declare the host functions
    pub fn declare_call(&mut self) -> Result<FuncRef> {
        let sig = {
            let mut sig = self.module.make_signature();
            sig.params.push(AbiParam::new(types::I32));
            sig.params.push(AbiParam::new(types::I64));
            sig.returns.push(AbiParam::new(types::I8));
            sig
        };

        // declare the host call function
        let host_id = self.module.declare_function(CALL, Linkage::Export, &sig)?;
        let local_id = self
            .module
            .declare_func_in_func(host_id, &mut self.ctx.func);
        Ok(local_id)
    }
}
