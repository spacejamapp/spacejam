//! SpaceJam PVM compiler

mod artifact;
mod compiler;
mod jit;
pub mod module;

pub use {
    compiler::Compiler,
    jit::JIT,
    module::{Context, ExecResult, ExtendedContext, Info, Module},
};
