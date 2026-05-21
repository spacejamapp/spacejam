//! Spacejam storage interfaces
//!
//! introduce read-only storage interface for the state.

pub use {
    archive::{Archive, ArchiveStorage},
    branch::Branch,
    commit::{Commit, Operation},
    kv::{KVStorage, MemoryDb},
    multitree::{MultiTreeStore, NewNode, NodeAddress, NodeRef},
    state::StateStorage,
    sync::SyncStorage,
};

mod archive;
mod branch;
mod commit;
mod kv;
mod multitree;
pub mod root;
mod state;
pub mod sync;

/// The column for the storage
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Column {
    /// The column for the state
    State = 0,

    /// The column for the sync
    Sync = 1,

    /// The column for the archive
    Archive = 2,

    /// The column for incremental trie nodes (parity-db multitree).
    TrieNodes = 3,
}

/// The storage of SpaceJam
pub trait Storage: StateStorage + SyncStorage + Send + Sync + 'static {}

impl<T: KVStorage> ArchiveStorage for T {}
impl<T: KVStorage> StateStorage for T {}
impl<T: KVStorage> SyncStorage for T {}
impl<T: KVStorage> Storage for T {}
