//! Compiled PVM module

use cranelift_codegen::ir::Function;
use std::collections::BTreeMap;

/// Compiled PVM module
pub struct Module {
    /// Functions mapped by PC
    _funcs: BTreeMap<u64, Function>,
}
