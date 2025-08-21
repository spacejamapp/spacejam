//! SpaceJam PVM compiler

mod compiler;
pub mod module;

pub use compiler::Compiler;
pub use module::{Context, ExecResult, ExtendedContext, Info, Module};
