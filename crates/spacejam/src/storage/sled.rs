//! Storage interface with sled

use anyhow::Result;
use score::{misc::OpaqueHash, state::Storage};
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

    fn batch_write(&self, kvs: Vec<(OpaqueHash, Vec<u8>)>) -> Result<()> {
        let mut batch = Batch::default();
        for (key, value) in kvs {
            batch.insert(key.as_ref(), value);
        }
        self.db.apply_batch(batch)?;
        Ok(())
    }

    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        Ok(self.db.get(key.as_ref())?.map(|v| v.to_vec()))
    }

    fn batch_read(&self, keys: Vec<OpaqueHash>) -> Result<Vec<Vec<u8>>> {
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
        let iter = self.db.scan_prefix(prefix.as_ref());

        Ok(iter.map(|r| {
            let (k, v) = r?;
            let key = OpaqueHash::try_from(k.to_vec())
                .map_err(|e| anyhow::anyhow!("failed to decode key: {e:?}"))?;
            Ok((key, v.to_vec()))
        }))
    }
}
