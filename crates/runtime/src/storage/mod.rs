//! Spacejam storage interfaces
//!
//! introduce read-only storage interface for the state.

pub use {
    branch::Branch,
    commit::{Commit, Operation},
    kv::{KVStorage, MemoryDb},
    state::StateStorage,
    sync::SyncStorage,
};

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

/// The storage of SpaceJam
pub trait Storage: StateStorage + SyncStorage {}

impl<T: KVStorage> StateStorage for T {}
impl<T: KVStorage> SyncStorage for T {}
impl<T: KVStorage> Storage for T {}
