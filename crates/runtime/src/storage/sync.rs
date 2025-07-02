//! Block storage

use anyhow::Result;
use score::{
    block::{Head, Header},
    Block, OpaqueHash,
};

use crate::storage::{Column, KVStorage};

/// The key for the sync storage
pub const SYNC: &[u8] = b"sync";

/// Sync storage
pub trait SyncStorage: KVStorage {
    /// Get the state from the storage
    fn sync_get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        self.get(Column::Sync, key)
    }

    /// Get the state from the storage
    fn sync_set(&self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<()> {
        self.set(Column::Sync, key, value)
    }

    /// Get the state from the storage
    fn sync_iter(&self) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        self.iter(Column::Sync)
    }

    fn sync_prefix_iter(
        &self,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        self.prefix_iter(Column::Sync, prefix)
    }

    fn sync_batch_read(&self, keys: Vec<Vec<u8>>) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        self.batch_read(Column::Sync, keys)
    }

    /// Get the descendant of the given hash in finalized chain.
    fn descendant(&self, parent: &OpaqueHash) -> Result<OpaqueHash> {
        let key = [SYNC, b"descendant", parent.as_ref()].concat();
        let value = self
            .sync_get(&key)?
            .ok_or(anyhow::anyhow!("Descendant not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Get the finalized head
    fn finalized(&self) -> Result<Head> {
        let key = [SYNC, b"finalized"].concat();
        let value = self
            .sync_get(&key)?
            .ok_or(anyhow::anyhow!("Finalized head not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Set the finalized head
    fn finalize(&self, head: &Head) -> Result<()> {
        let key = [SYNC, b"finalized"].concat();
        self.sync_set(key, codec::encode(head)?)?;

        // set the descendant of the parent
        let parent = self
            .parent(&head.hash)?
            .ok_or(anyhow::anyhow!("Parent not found"))?;
        let key = [SYNC, b"descendant", parent.as_ref()].concat();
        self.sync_set(key, head.hash)?;
        Ok(())
    }

    /// Get the header
    fn header(&self, hash: &OpaqueHash) -> Result<Header> {
        let key = [SYNC, b"header", hash.as_ref()].concat();
        let value = self
            .sync_get(&key)?
            .ok_or(anyhow::anyhow!("Header not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Get the parent
    fn parent(&self, block: &OpaqueHash) -> Result<Option<OpaqueHash>> {
        let key = [SYNC, b"parent", block.as_ref()].concat();
        let value = self.sync_get(&key)?;
        if let Some(value) = value {
            Ok(Some(codec::decode(value.as_ref())?))
        } else {
            Ok(None)
        }
    }

    /// Get the state root
    fn state_root(&self, block: &OpaqueHash) -> Result<Option<OpaqueHash>> {
        let mut key = [SYNC, b"state_root"].concat();
        key.extend_from_slice(block.as_ref());
        let value = self.sync_get(&key)?;
        if let Some(value) = value {
            Ok(Some(codec::decode(value.as_ref())?))
        } else {
            Ok(None)
        }
    }

    /// Get the beefy root
    fn beefy_root(&self, block: &OpaqueHash) -> Result<OpaqueHash> {
        let mut key = [SYNC, b"beefy_root"].concat();
        key.extend_from_slice(block.as_ref());
        let value = self
            .sync_get(&key)?
            .ok_or(anyhow::anyhow!("Beefy root not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Fetch the blocks
    fn fetch_blocks(&self, hashes: &[OpaqueHash]) -> Result<Vec<Block>> {
        Ok(self
            .sync_batch_read(
                hashes
                    .iter()
                    .map(|hash| [SYNC, hash.as_ref()].concat())
                    .collect::<Vec<_>>(),
            )?
            .into_iter()
            .filter_map(|(_, value)| codec::decode(&value).ok())
            .collect::<Vec<_>>())
    }
}
