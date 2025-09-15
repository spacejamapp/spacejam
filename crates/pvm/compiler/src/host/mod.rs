//! Host call trampoline

pub use abi::*;
use anyhow::Result;
use cranelift::{
    codegen::ir::{FuncRef, Function},
    jit::JITBuilder,
    module::{self, Linkage},
    prelude::Signature,
};
use pvm::Argument;
use std::collections::BTreeMap;

pub const CALL: &str = "call";
pub const SBRK: &str = "sbrk";
pub const MGET: &str = "mget";
pub const MSET: &str = "mset";

mod abi;
mod sig;

/// Register host call symbols
pub fn symbols<X: Argument>(builder: &mut JITBuilder) {
    builder.symbol(CALL, abi::ecalli::<X> as *const u8);
    builder.symbol(SBRK, abi::sbrk::<X> as *const u8);
    builder.symbol(MGET, abi::mget::<X> as *const u8);
    builder.symbol(MSET, abi::mset::<X> as *const u8);
}

/// Declare the host functions
pub fn declare(
    module: &mut impl module::Module,
    func: &mut Function,
) -> Result<BTreeMap<&'static str, FuncRef>> {
    let mut map = BTreeMap::new();
    for (name, sig) in self::host_calls() {
        let id = module.declare_function(name, Linkage::Import, &sig)?;
        let funref = module.declare_func_in_func(id, func);
        map.insert(name, funref);
    }
    Ok(map)
}

fn host_calls() -> BTreeMap<&'static str, Signature> {
    let mut map = BTreeMap::new();
    map.insert(CALL, sig::ECALLI.clone());
    map.insert(SBRK, sig::SBRK.clone());
    map.insert(MGET, sig::MGET.clone());
    map.insert(MSET, sig::MSET.clone());
    map
}
