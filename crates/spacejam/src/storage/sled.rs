//! Storage interface with sled
#![cfg(feature = "sled")]

use anyhow::Result;
use score::runtime::Storage;
use sled::{Batch, Db};
use std::path::Path;

/// Sled storage
pub struct Sled {
    db: Db,
}

impl Storage for Sled {
    fn open(path: impl AsRef<Path>) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    fn set(&self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<()> {
        self.db.insert(key.as_ref(), value.as_ref())?;
        Ok(())
    }

    fn batch_write(&self, kvs: Vec<(Vec<u8>, Vec<u8>)>) -> Result<()> {
        let mut batch = Batch::default();
        for (key, value) in kvs {
            batch.insert(key, value);
        }
        self.db.apply_batch(batch)?;
        Ok(())
    }

    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        Ok(self.db.get(key.as_ref())?.map(|v| v.to_vec()))
    }

    fn remove(&self, key: impl AsRef<[u8]>) -> Result<()> {
        let _ = self.db.remove(key.as_ref())?;
        Ok(())
    }

    fn prefix_iter(
        &self,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        let iter = self.db.scan_prefix(prefix.as_ref());

        Ok(iter.map(|r| {
            let (k, v) = r?;
            Ok((k.to_vec(), v.to_vec()))
        }))
    }
}
