//! The storage of SpaceJam

pub mod parity;

pub use parity::Parity;
use runtime::storage::KVStorage;
pub use runtime::storage::MemoryDb;
use std::path::PathBuf;

/// Open the database
pub fn open(path: PathBuf) -> anyhow::Result<impl KVStorage> {
    Parity::try_from(path)
}
