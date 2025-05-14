//! Spacejam storage interfaces
//!
//! introduce read-only storage interface for the state.

pub use {
    kv::{KVStorage, MemoryDb},
    state::Storage,
    sync::SyncStorage,
};

mod kv;
mod state;
pub mod sync;

impl<T: KVStorage> Storage for T {}
impl<T: KVStorage> SyncStorage for T {}
