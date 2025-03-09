//! Branch storage

use crate::runtime::{storage::KVStorage, Head};
use anyhow::Result;

/// A branch of the blockchain
///
/// maybe use column in rocksdb instead in the future.
#[derive(Debug, Clone)]
pub struct Branch<'s, S: KVStorage> {
    /// The latest finalized head.
    head: Head,

    /// The finalized storage
    pub finalized: &'s S,
}

impl<'s, S: KVStorage> Branch<'s, S> {
    /// Create a new branch
    pub fn checkout(finalized: &'s S, head: Head) -> Self {
        Self { finalized, head }
    }

    /// Drop the branch, clean all the data in the branch
    pub fn drop(&self) -> Result<()> {
        let mut iter = self.finalized.prefix_iter(self.head.hash[..6].as_ref())?;
        while let Some(Ok((key, _))) = iter.next() {
            self.finalized.remove(key)?;
        }
        Ok(())
    }

    /// Finalize the branch
    ///
    /// Override all storage to the finalized chain.
    pub fn finalize(&self) -> Result<()> {
        let mut iter = self.finalized.prefix_iter(self.head.hash[..6].as_ref())?;
        while let Some(Ok((key, value))) = iter.next() {
            self.finalized.set(key, value)?;
        }
        Ok(())
    }

    /// Wrap the key with the branch prefix
    fn wrap(&self, key: impl AsRef<[u8]>) -> Vec<u8> {
        [self.head.hash[..6].as_ref(), key.as_ref()].concat()
    }

    /// Unwrap the key from the branch prefix
    pub fn unwrap(&self, key: impl AsRef<[u8]>) -> Vec<u8> {
        let key = key.as_ref();
        if key.len() > 6 && key[..6] == self.head.hash[..6] {
            key[6..].to_vec()
        } else {
            key.to_vec()
        }
    }
}

impl<S: KVStorage> KVStorage for Branch<'_, S> {
    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        if let Ok(Some(value)) = self.finalized.get(self.wrap(&key)) {
            Ok(Some(value))
        } else {
            self.finalized.get(key)
        }
    }

    fn set(&self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<()> {
        self.finalized.set(self.wrap(key), value)
    }

    fn batch_write(&self, kvs: Vec<(Vec<u8>, Vec<u8>)>) -> Result<()> {
        self.finalized.batch_write(kvs)
    }

    fn remove(&self, key: impl AsRef<[u8]>) -> Result<()> {
        self.finalized.remove(self.wrap(key))
    }

    fn prefix_iter(
        &self,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        self.finalized.prefix_iter(self.wrap(&prefix))
    }

    fn batch_read(&self, keys: Vec<Vec<u8>>) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let mut results = Vec::new();

        // Try to read each key from the branch storage first
        for key in &keys {
            results.push((key.clone(), self.get(key)?.unwrap_or_default()));
        }

        Ok(results)
    }

    fn is_empty(&self) -> bool {
        self.finalized.is_empty()
    }
}
