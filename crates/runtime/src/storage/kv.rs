//! Key-value storage abstraction

use crate::storage::{Column, Commit, MultiTree, NewNode, NodeAddress};
use anyhow::Result;
use crypto::merkle::multitree::MultiTreeMap;
use score::{OpaqueHash, TrieKey, state::StateKeyLike};
use std::{
    collections::BTreeMap,
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

/// In-memory key-value storage.
#[derive(Default)]
pub struct MemoryDb {
    data: Arc<RwLock<BTreeMap<TrieKey, Vec<u8>>>>,
    tries: MultiTreeMap,
}

impl MemoryDb {
    /// Run a closure under the data read lock.
    pub fn with_data<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&BTreeMap<TrieKey, Vec<u8>>) -> R,
    {
        Ok(f(&*self
            .data
            .read()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?))
    }

    /// Reset the memory database
    pub fn reset(&self, data: BTreeMap<TrieKey, Vec<u8>>) {
        *self.data.write().unwrap() = data;
    }
}

impl KVStorage for MemoryDb {
    fn commit(&self, _column: Column, commit: Commit<TrieKey, Vec<u8>>) -> Result<()> {
        let mut data = self
            .data
            .write()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;

        for (key, value) in commit.iset() {
            data.insert(*key, value.clone());
        }

        for key in commit.iremoval() {
            data.remove(key);
        }

        Ok(())
    }

    fn set(&self, _column: Column, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<()> {
        let mut data = self
            .data
            .write()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;

        data.insert(key.as_ref().as_state_key(), value.as_ref().to_vec());
        Ok(())
    }

    fn get(&self, _column: Column, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        let data = self
            .data
            .read()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;
        Ok(data.get(&key.as_ref().as_state_key()).cloned())
    }

    fn iter(&self, _column: Column) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        let data = self
            .data
            .read()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;

        // Clone all entries to avoid holding the lock during iteration
        let entries: Vec<(Vec<u8>, Vec<u8>)> =
            data.iter().map(|(k, v)| (k.to_vec(), v.clone())).collect();

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
        let prefix_bytes = prefix.as_ref();

        // Clone all matching entries to avoid holding the lock during iteration
        let matches: Vec<(Vec<u8>, Vec<u8>)> = data
            .iter()
            .filter(|(k, _)| k.starts_with(prefix_bytes))
            .map(|(k, v)| (k.to_vec(), v.clone()))
            .collect();

        Ok(matches.into_iter().map(Ok))
    }
}

impl MultiTree for MemoryDb {
    fn insert_tree(&self, key: OpaqueHash, root: NewNode) -> Result<()> {
        self.tries.insert_tree(key, root)
    }

    fn dereference_tree(&self, key: OpaqueHash) -> Result<()> {
        self.tries.dereference_tree(key)
    }

    fn get_root(&self, key: OpaqueHash) -> Result<Option<(Vec<u8>, Vec<NodeAddress>)>> {
        self.tries.get_root(key)
    }

    fn get_node(&self, address: NodeAddress) -> Result<Option<(Vec<u8>, Vec<NodeAddress>)>> {
        self.tries.get_node(address)
    }
}
