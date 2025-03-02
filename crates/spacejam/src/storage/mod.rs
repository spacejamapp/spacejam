//! The storage of SpaceJam

pub mod rocksdb;
pub mod sled;

pub use score::runtime::storage::MemoryDb;

#[cfg(feature = "rocksdb")]
pub use rocksdb::RocksDB;

#[cfg(feature = "sled")]
pub use sled::Sled;
