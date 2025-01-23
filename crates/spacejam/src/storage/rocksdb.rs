//! The RocksDB storage of SpaceJam
#![cfg(feature = "rocksdb")]
use anyhow::Result;
use rocksdb::{WriteBatch, DB};
use score::{state::Storage, OpaqueHash};
use std::path::Path;

/// The RocksDB storage of SpaceJam
pub struct RocksDB {
    db: DB,
    branch: Option<OpaqueHash>,
}

impl Storage for RocksDB {
    fn open(path: impl AsRef<Path>) -> Result<Self> {
        let db = DB::open_default(path.as_ref())?;
        Ok(Self { db, branch: None })
    }

    fn branch(&self) -> Option<OpaqueHash> {
        self.branch
    }

    fn checkout(&mut self, branch: Option<OpaqueHash>) {
        self.branch = branch;
    }

    fn set(&self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<()> {
        self.db.put(key.as_ref(), value.as_ref())?;
        Ok(())
    }

    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        Ok(self.db.get(key.as_ref())?.map(|v| v.to_vec()))
    }

    fn batch_write(&self, kvs: Vec<(OpaqueHash, Vec<u8>)>) -> Result<()> {
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

    fn batch_read(&self, keys: Vec<[u8; 32]>) -> Result<Vec<Vec<u8>>> {
        let values = keys
            .iter()
            .map(|k| self.get(k).map(|v| v.unwrap_or_default()))
            .collect::<Result<Vec<_>>>()?;
        Ok(values)
    }

    fn prefix_iter(
        &self,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(OpaqueHash, Vec<u8>)>>> {
        let iter = self.db.prefix_iterator(prefix.as_ref());
        Ok(iter.map(|r| {
            let (k, v) = r?;
            Ok((OpaqueHash::try_from(k.to_vec()).unwrap(), v.to_vec()))
        }))
    }
}
