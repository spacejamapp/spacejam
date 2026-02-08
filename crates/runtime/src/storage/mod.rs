//! Spacejam storage interfaces
//!
//! introduce read-only storage interface for the state.

pub use {
    archive::{Archive, ArchiveStorage},
    branch::Branch,
    commit::{Commit, Operation},
    kv::{KVStorage, MemoryDb},
    state::StateStorage,
    sync::SyncStorage,
};

mod archive;
mod branch;
mod commit;
mod kv;
pub mod root;
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

    /// The column for the archive
    Archive = 2,
}

/// The storage of SpaceJam
pub trait Storage: StateStorage + SyncStorage + Send + Sync + 'static {}

impl<T: KVStorage> ArchiveStorage for T {}
impl<T: KVStorage> StateStorage for T {}
impl<T: KVStorage> SyncStorage for T {}
impl<T: KVStorage> Storage for T {}
