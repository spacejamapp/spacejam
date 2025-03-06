//! Spacejam storage interfaces

pub use {
    block::BlockStorage,
    kv::{KVStorage, MemoryDb},
    state::Storage,
};

mod block;
mod kv;
mod state;

impl<T: KVStorage> Storage for T {}
impl<T: KVStorage> BlockStorage for T {}
