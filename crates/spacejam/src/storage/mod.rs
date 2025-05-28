//! The storage of SpaceJam

pub mod parity;
pub mod rocksdb;
pub mod sled;

use std::path::PathBuf;

use runtime::storage::KVStorage;
pub use runtime::storage::MemoryDb;

#[cfg(feature = "rocksdb")]
pub use rocksdb::RocksDB;

#[cfg(feature = "parity")]
pub use parity::Parity;

#[cfg(feature = "sled")]
pub use sled::Sled;

/// Open the database
pub fn open(path: PathBuf) -> anyhow::Result<impl KVStorage> {
    #[cfg(feature = "parity")]
    {
        Parity::try_from(path)
    }

    #[cfg(feature = "sled")]
    {
        Ok(Sled::try_from(buf)?)
    }

    #[cfg(feature = "rocksdb")]
    {
        Ok(RocksDB::try_from(buf)?)
    }
}
