//! SpaceJam PVM compiler

mod artifact;
mod compiler;
mod jit;
mod memory;
pub mod module;
pub mod trap;

pub use {
    compiler::Compiler,
    jit::JIT,
    memory::Memory,
    module::{Context, ExecResult, Info, Module},
    trap::TrapInfo,
};
