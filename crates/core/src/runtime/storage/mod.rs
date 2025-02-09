pub use {branch::Branch, kv::KVStorage, state::Storage};

mod branch;
mod kv;
mod state;

impl<T: KVStorage> Storage for T {}
