//! Spacejam storage interfaces
//!
//! introduce read-only storage interface for the state.

pub use {
    archive::ArchiveStorage,
    branch::Branch,
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

/// The column for the storage
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Column {
    /// The column for the sync
    State = 0,

    /// The column for the state
    Sync = 1,
}

impl<T: KVStorage> Storage for T {}
impl<T: KVStorage> SyncStorage for T {}
