//! Host call trampoline

use std::collections::BTreeMap;

use crate::JIT;
use anyhow::Result;
use cranelift::prelude::{types, AbiParam, Signature};
use cranelift_codegen::ir::FuncRef;
use cranelift_jit::JITBuilder;
use cranelift_module::{Linkage, Module};
use pvm::Argument;
pub use {abi::*, value::Value};

pub const CALL: &str = "call";
pub const SBRK: &str = "sbrk";
pub const MGET: &str = "mget";
pub const MSET: &str = "mset";

mod abi;
mod value;

/// Register host call symbols
pub fn symbols<X: Argument>(builder: &mut JITBuilder) {
    builder.symbol(CALL, abi::call::<X> as *const u8);
    builder.symbol(SBRK, abi::sbrk::<X> as *const u8);
}

impl JIT {
    /// Create new JIT module builder for host functions
    /// Declare the host functions
    pub fn declare_host(&mut self) -> Result<BTreeMap<&'static str, FuncRef>> {
        let mut map = BTreeMap::new();
        map.insert(CALL, self.declare_call()?);
        map.insert(SBRK, self.declare_sbrk()?);
        map.insert(MGET, self.declare_mget()?);
        map.insert(MSET, self.declare_mset()?);
        Ok(map)
    }

    /// Declare a function
    fn declare(&mut self, name: &str, sig: Signature) -> Result<FuncRef> {
        let host_id = self.module.declare_function(name, Linkage::Import, &sig)?;
        let local_id = self
            .module
            .declare_func_in_func(host_id, &mut self.ctx.func);
        Ok(local_id)
    }

    /// Declare the host functions
    fn declare_call(&mut self) -> Result<FuncRef> {
        let sig = {
            let mut sig = self.module.make_signature();
            sig.params.push(AbiParam::new(types::I32));
            sig.params.push(AbiParam::new(types::I64));
            sig.returns.push(AbiParam::new(types::I8));
            sig
        };

        // declare the host call function
        self.declare(CALL, sig)
    }

    /// Declare the mget function
    fn declare_mget(&mut self) -> Result<FuncRef> {
        let sig = {
            let mut sig = self.module.make_signature();
            sig.params.push(AbiParam::new(types::I64));
            sig.params.push(AbiParam::new(types::I32));
            sig.params.push(AbiParam::new(types::I32));
            sig.params.push(AbiParam::new(types::I8));
            sig.returns.push(AbiParam::new(types::I64));
            sig
        };

        self.declare(MGET, sig)
    }

    /// Declare the mset function
    fn declare_mset(&mut self) -> Result<FuncRef> {
        let sig = {
            let mut sig = self.module.make_signature();
            sig.params.push(AbiParam::new(types::I64));
            sig.params.push(AbiParam::new(types::I32));
            sig.params.push(AbiParam::new(types::I32));
            sig.params.push(AbiParam::new(types::I8));
            sig.returns.push(AbiParam::new(types::I64));
            sig
        };

        self.declare(MSET, sig)
    }

    /// Declare the sbrk function
    fn declare_sbrk(&mut self) -> Result<FuncRef> {
        let sig = {
            let mut sig = self.module.make_signature();
            sig.params.push(AbiParam::new(types::I64));
            sig.params.push(AbiParam::new(types::I8));
            sig.params.push(AbiParam::new(types::I8));
            sig
        };

        self.declare(SBRK, sig)
    }
}
