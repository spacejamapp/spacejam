//! Key-value storage abstraction

use crate::storage::{Column, Commit};
use anyhow::Result;
use score::TrieKey;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

/// Key-value storage
pub trait KVStorage: Send + Sync + 'static {
    /// Batch write a set of key-value pairs to the storage
    fn commit(&self, column: Column, commit: Commit<TrieKey, Vec<u8>>) -> Result<()>;

    /// Set a key-value pair with column specified
    fn set(&self, column: Column, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<()>;

    /// Get a value from the storage with column specified
    fn get(&self, column: Column, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>>;

    /// Iterate over the storage with column specified
    fn iter(&self, column: Column) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>>;

    /// Iterate over the storage with a prefix and column specified
    fn prefix_iter(
        &self,
        column: Column,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>>;

    /// Batch read a set of key-value pairs from the storage with column specified
    fn batch_read(&self, column: Column, keys: Vec<Vec<u8>>) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        keys.iter()
            .map(|key| {
                self.get(column, key)
                    .map(|v| (key.to_vec(), v.unwrap_or_default()))
            })
            .collect::<Result<Vec<_>>>()
    }
}

/// In-memory key-value storage implementation
///
/// This implementation stores all data in memory and is not persistent.
/// It's useful for testing and for situations where persistence isn't required.
#[derive(Default)]
pub struct MemoryDb {
    data: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl MemoryDb {
    /// Deep clone the memory database
    pub fn deep_clone(&self) -> HashMap<Vec<u8>, Vec<u8>> {
        self.data.read().unwrap().clone()
    }

    /// Duplicate the memory database
    pub fn dup(&self) -> Self {
        Self {
            data: Arc::new(RwLock::new(self.data.read().unwrap().clone())),
        }
    }

    /// Reset the memory database
    pub fn reset(&self, data: HashMap<Vec<u8>, Vec<u8>>) {
        let mut curr = self.data.write().unwrap();
        *curr = data;
    }
}

impl KVStorage for MemoryDb {
    fn commit(&self, _column: Column, commit: Commit<TrieKey, Vec<u8>>) -> Result<()> {
        let mut data = self
            .data
            .write()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;

        for (key, value) in commit.iset() {
            data.insert(key.to_vec(), value.clone());
        }

        for key in commit.iremoval() {
            data.remove(key.as_ref());
        }

        Ok(())
    }

    fn set(&self, _column: Column, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<()> {
        let mut data = self
            .data
            .write()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;

        data.insert(key.as_ref().to_vec(), value.as_ref().to_vec());
        Ok(())
    }

    fn get(&self, _column: Column, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        let data = self
            .data
            .read()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;
        Ok(data.get(key.as_ref()).cloned())
    }

    fn iter(&self, _column: Column) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        let data = self
            .data
            .read()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;

        // Clone all entries to avoid holding the lock during iteration
        let entries: Vec<(Vec<u8>, Vec<u8>)> =
            data.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

        Ok(entries.into_iter().map(Ok))
    }

    fn prefix_iter(
        &self,
        _column: Column,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        let data = self
            .data
            .read()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;
        let prefix_bytes = prefix.as_ref().to_vec();

        // Clone all matching entries to avoid holding the lock during iteration
        let matches: Vec<(Vec<u8>, Vec<u8>)> = data
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix_bytes))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        Ok(matches.into_iter().map(Ok))
    }
}
