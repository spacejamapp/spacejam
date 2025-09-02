//! Host call trampoline

use crate::JIT;
pub use abi::*;
use anyhow::Result;
use cranelift::prelude::{types, AbiParam, Signature};
use cranelift_codegen::ir::FuncRef;
use cranelift_jit::JITBuilder;
use cranelift_module::{FuncId, Linkage, Module};
use pvm::Argument;
use std::collections::BTreeMap;

pub const CALL: &str = "call";
pub const SBRK: &str = "sbrk";
pub const MGET: &str = "mget";
pub const MSET: &str = "mset";

mod abi;

/// Register host call symbols
pub fn symbols<X: Argument>(builder: &mut JITBuilder) {
    builder.symbol(CALL, abi::call::<X> as *const u8);
    builder.symbol(SBRK, abi::sbrk::<X> as *const u8);
    builder.symbol(MGET, abi::mget::<X> as *const u8);
    builder.symbol(MSET, abi::mset::<X> as *const u8);
}

impl JIT {
    /// Declare the host functions
    pub fn declare_host_in_func(
        &mut self,
        host: BTreeMap<String, FuncId>,
    ) -> Result<BTreeMap<String, FuncRef>> {
        let mut map = BTreeMap::new();
        for (name, id) in host {
            let func = self.module.declare_func_in_func(id, &mut self.ctx.func);
            map.insert(name, func);
        }
        Ok(map)
    }

    /// Declare the host functions in the module
    pub fn declare_host_in_module(&mut self) -> Result<BTreeMap<String, FuncId>> {
        let mut map = BTreeMap::new();
        for (name, sig) in self.host_calls() {
            let id = self.module.declare_function(&name, Linkage::Import, &sig)?;
            map.insert(name, id);
        }
        Ok(map)
    }

    fn host_calls(&self) -> BTreeMap<String, Signature> {
        let mut map = BTreeMap::new();
        map.insert(CALL.to_string(), {
            let mut sig = self.module.make_signature();
            sig.params.push(AbiParam::new(types::I32));
            sig.params.push(AbiParam::new(types::I64));
            sig.returns.push(AbiParam::new(types::I8));
            sig
        });
        map.insert(SBRK.to_string(), {
            let mut sig = self.module.make_signature();
            sig.params.push(AbiParam::new(types::I64));
            sig.params.push(AbiParam::new(types::I8));
            sig.params.push(AbiParam::new(types::I8));
            sig
        });
        map.insert(MGET.to_string(), {
            let mut sig = self.module.make_signature();
            sig.params.push(AbiParam::new(types::I64));
            sig.params.push(AbiParam::new(types::I64));
            sig.params.push(AbiParam::new(types::I8));
            sig.returns.push(AbiParam::new(types::I64));
            sig
        });
        map.insert(MSET.to_string(), {
            let mut sig = self.module.make_signature();
            sig.params.push(AbiParam::new(types::I64));
            sig.params.push(AbiParam::new(types::I64));
            sig.params.push(AbiParam::new(types::I64));
            sig.params.push(AbiParam::new(types::I8));
            sig
        });
        map
    }
}
