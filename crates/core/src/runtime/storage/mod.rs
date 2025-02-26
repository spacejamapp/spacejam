//! Spacejam storage interfaces

pub use {block::BlockStorage, branch::Branch, kv::KVStorage, state::Storage};

mod block;
mod branch;
mod kv;
mod state;

impl<T: KVStorage> Storage for T {}
impl<T: KVStorage> BlockStorage for T {}
