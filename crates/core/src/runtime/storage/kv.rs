//! Key-value storage abstraction

use crate::{state::key, OpaqueHash};
use anyhow::Result;

/// Key-value storage
pub trait KVStorage {
    /// Set a value in the storage
    fn set(&self, _key: impl AsRef<[u8]>, _value: impl AsRef<[u8]>) -> Result<()>;

    /// Batch write a set of key-value pairs to the storage
    fn batch_write(&self, kvs: Vec<(Vec<u8>, Vec<u8>)>) -> Result<()>;

    /// Get a value from the storage
    fn get(&self, _key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>>;

    /// Remove a key-value pair from the storage
    fn remove(&self, key: impl AsRef<[u8]>) -> Result<()>;

    /// Iterate over the storage with a prefix
    fn prefix_iter(
        &self,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>>;

    /// Batch read a set of key-value pairs from the storage
    fn batch_read(&self, keys: Vec<OpaqueHash>) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        keys.iter()
            .map(|key| self.get(key).map(|v| (key.to_vec(), v.unwrap_or_default())))
            .collect::<Result<Vec<_>>>()
    }

    /// Check if the storage is empty
    fn is_empty(&self) -> bool {
        self.get(key::TIMESLOT).map(|v| v.is_none()).unwrap_or(true)
    }
}
