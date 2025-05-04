//! The RocksDB storage of SpaceJam
#![cfg(feature = "rocksdb")]

use anyhow::Result;
use rocksdb::{WriteBatch, DB};
use runtime::storage::KVStorage;

/// The RocksDB storage of SpaceJam
pub struct RocksDB {
    db: DB,
}

impl KVStorage for RocksDB {
    fn set(&self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<()> {
        self.db.put(key.as_ref(), value.as_ref())?;
        Ok(())
    }

    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        Ok(self.db.get(key.as_ref())?.map(|v| v.to_vec()))
    }

    fn batch_write(&self, kvs: Vec<(Vec<u8>, Vec<u8>)>) -> Result<()> {
        let mut batch = WriteBatch::default();
        for (key, value) in kvs {
            batch.put(&key, &value);
        }
        self.db.write(batch)?;
        Ok(())
    }

    fn remove(&self, key: impl AsRef<[u8]>) -> Result<()> {
        self.db.delete(key.as_ref()).map_err(Into::into)
    }

    fn prefix_iter(
        &self,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        let iter = self.db.prefix_iterator(prefix.as_ref());
        Ok(iter.map(|r| {
            let (k, v) = r?;
            Ok((k.to_vec(), v.to_vec()))
        }))
    }
}
