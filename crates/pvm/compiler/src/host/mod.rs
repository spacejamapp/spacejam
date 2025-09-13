//! Host call trampoline

use crate::Compiler;
pub use abi::*;
use anyhow::Result;
use cranelift::prelude::Signature;
use cranelift_codegen::ir::{FuncRef, Function};
use cranelift_jit::JITBuilder;
use cranelift_module::{FuncId, Linkage, Module};
use pvm::Argument;
use std::collections::BTreeMap;
pub use translator::host as sig;

pub const CALL: &str = "call";
pub const SBRK: &str = "sbrk";
pub const MGET: &str = "mget";
pub const MSET: &str = "mset";

mod abi;

/// Register host call symbols
pub fn symbols<X: Argument>(builder: &mut JITBuilder) {
    builder.symbol(CALL, abi::ecalli::<X> as *const u8);
    builder.symbol(SBRK, abi::sbrk::<X> as *const u8);
    builder.symbol(MGET, abi::mget::<X> as *const u8);
    builder.symbol(MSET, abi::mset::<X> as *const u8);
}

/// Get the host call table
pub fn table<X: Argument>() -> *const u8 {
    let table = [
        abi::ecalli::<X> as *const u8,
        abi::sbrk::<X> as *const u8,
        abi::mget::<X> as *const u8,
        abi::mset::<X> as *const u8,
    ];
    table.as_ptr() as *const u8
}

impl Compiler {
    /// Declare the host functions
    pub fn declare_host_in_func(
        &mut self,
        host: BTreeMap<String, FuncId>,
        func: &mut Function,
    ) -> Result<BTreeMap<String, FuncRef>> {
        let mut map = BTreeMap::new();
        for (name, id) in host {
            let func = self.module.declare_func_in_func(id, func);
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
        map.insert(CALL.to_string(), sig::ECALLI.clone());
        map.insert(SBRK.to_string(), sig::SBRK.clone());
        map.insert(MGET.to_string(), sig::MGET.clone());
        map.insert(MSET.to_string(), sig::MSET.clone());
        map
    }
}
