//! SpaceJam PVM compiler

pub use {
    artifact::{Artifact, SPACEVM_CACHE_DIR},
    compiler::Compiler,
    cranelift_codegen::timing,
    memory::Memory,
    module::{Info, Module},
    pvm::MemoryLike,
    translator::Translator,
    trap::TrapInfo,
};

mod artifact;
mod compiler;
pub mod engine;
pub mod host;
pub mod memory;
pub mod module;
pub mod object;
mod translate;
pub mod trap;
