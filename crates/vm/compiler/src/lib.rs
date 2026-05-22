//! SpaceJam PVM compiler

pub use {
    artifact::{Artifact, SPACEJAM_CACHE_DIR},
    compiler::Compiler,
    cranelift_codegen::timing,
    engine::Engine,
    exec::Executable,
    memory::Memory,
    module::{JITModule, ModuleLike, ObjectModule},
    pvm::MemoryLike,
    translator::Translator,
    trap::TrapInfo,
};

mod artifact;
mod compiler;
pub mod engine;
mod exec;
pub mod host;
pub mod memory;
pub mod module;
pub mod numa;
pub mod trap;

#[cfg(target_os = "macos")]
pub type Module = crate::JITModule;

#[cfg(not(target_os = "macos"))]
pub type Module = crate::ObjectModule;
