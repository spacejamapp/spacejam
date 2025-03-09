//! Key-value storage abstraction

use crate::{state::key, TimeSlot};
use anyhow::Result;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

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
    fn batch_read(&self, keys: Vec<Vec<u8>>) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        keys.iter()
            .map(|key| self.get(key).map(|v| (key.to_vec(), v.unwrap_or_default())))
            .collect::<Result<Vec<_>>>()
    }

    /// Check if the storage is empty
    fn is_empty(&self) -> bool {
        let timeslot = self.get(key::TIMESLOT);
        if let Ok(Some(timeslot)) = timeslot {
            codec::decode::<TimeSlot>(timeslot.as_ref()).is_err()
        } else {
            true
        }
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

impl KVStorage for MemoryDb {
    fn set(&self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<()> {
        let mut data = self
            .data
            .write()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;
        data.insert(key.as_ref().to_vec(), value.as_ref().to_vec());
        Ok(())
    }

    fn batch_write(&self, kvs: Vec<(Vec<u8>, Vec<u8>)>) -> Result<()> {
        let mut data = self
            .data
            .write()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;
        for (key, value) in kvs {
            data.insert(key, value);
        }
        Ok(())
    }

    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        let data = self
            .data
            .read()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;
        Ok(data.get(key.as_ref()).cloned())
    }

    fn remove(&self, key: impl AsRef<[u8]>) -> Result<()> {
        let mut data = self
            .data
            .write()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;
        data.remove(key.as_ref());
        Ok(())
    }

    fn prefix_iter(
        &self,
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

    fn is_empty(&self) -> bool {
        match self.data.read() {
            Ok(data) => data.is_empty(),
            Err(_) => true, // Consider poisoned lock as empty
        }
    }
}
