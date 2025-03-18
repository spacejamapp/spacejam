//! Spacejam storage interfaces

pub use {
    branch::Branch,
    kv::{KVStorage, MemoryDb},
    state::Storage,
    sync::SyncStorage,
};

mod branch;
mod kv;
mod state;
mod sync;

impl<T: KVStorage> Storage for T {}
impl<T: KVStorage> SyncStorage for T {}
