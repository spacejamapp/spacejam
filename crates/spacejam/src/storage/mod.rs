//! The storage of SpaceJam

pub mod parity;

use runtime::storage::KVStorage;
pub use runtime::storage::MemoryDb;
use std::path::PathBuf;

#[cfg(feature = "parity")]
pub use parity::Parity;

/// Open the database
pub fn open(path: PathBuf) -> anyhow::Result<impl KVStorage> {
    #[cfg(feature = "parity")]
    {
        Parity::try_from(path)
    }
}
