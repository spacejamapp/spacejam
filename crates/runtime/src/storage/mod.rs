//! Spacejam storage interfaces
//!
//! introduce read-only storage interface for the state.

pub use {
    archive::ArchiveStorage,
    commit::{Commit, Operation},
    kv::{KVStorage, MemoryDb},
    state::Storage,
    sync::SyncStorage,
};

mod archive;
mod branch;
mod commit;
mod kv;
mod state;
pub mod sync;

impl<T: KVStorage> Storage for T {}
impl<T: KVStorage> SyncStorage for T {}
