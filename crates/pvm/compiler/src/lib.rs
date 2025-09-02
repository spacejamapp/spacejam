//! SpaceJam PVM compiler

pub use {
    artifact::{Artifact, JASTIME_CACHE_DIR},
    compiler::Compiler,
    cranelift_codegen::timing,
    jit::JIT,
    memory::Memory,
    module::{Info, Module},
    pvm::MemoryLike,
    trap::TrapInfo,
};

mod artifact;
mod compiler;
pub mod engine;
pub mod host;
mod jit;
pub mod memory;
pub mod module;
pub mod trap;
