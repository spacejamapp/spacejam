//! Block storage

use anyhow::Result;
use score::{
    block::{Head, Header},
    Block, OpaqueHash,
};

use crate::storage::{Column, KVStorage};

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
        let key = Key::Descendant(parent.clone()).key();
        let value = self
            .sync_get(&key)?
            .ok_or(anyhow::anyhow!("Descendant not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Get the finalized head
    fn finalized(&self) -> Result<Head> {
        let key = Key::Finalized.key();
        let value = self
            .sync_get(&key)?
            .ok_or(anyhow::anyhow!("Finalized head not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Set the finalized head
    fn finalize(&self, head: &Head) -> Result<()> {
        let key = Key::Finalized.key();
        self.sync_set(key, codec::encode(head)?)?;

        // set the descendant of the parent
        let parent = self
            .parent(&head.hash)?
            .ok_or(anyhow::anyhow!("Parent not found"))?;
        let key = Key::Descendant(parent).key();
        self.sync_set(key, head.hash)?;
        Ok(())
    }

    /// Get the header
    fn header(&self, hash: &OpaqueHash) -> Result<Header> {
        let key = Key::Header(hash.clone()).key();
        let value = self
            .sync_get(&key)?
            .ok_or(anyhow::anyhow!("Header not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Get the parent
    fn parent(&self, block: &OpaqueHash) -> Result<Option<OpaqueHash>> {
        let key = Key::Parent(block.clone()).key();
        let value = self.sync_get(&key)?;
        if let Some(value) = value {
            Ok(Some(codec::decode(value.as_ref())?))
        } else {
            Ok(None)
        }
    }

    /// Get the state root
    fn state_root(&self, block: &OpaqueHash) -> Result<Option<OpaqueHash>> {
        let key = Key::StateRoot(block.clone()).key();
        let value = self.sync_get(&key)?;
        if let Some(value) = value {
            Ok(Some(codec::decode(value.as_ref())?))
        } else {
            Ok(None)
        }
    }

    /// Get the beefy root
    fn beefy_root(&self, block: &OpaqueHash) -> Result<OpaqueHash> {
        let key = Key::BeefyRoot(block.clone()).key();
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
                    .map(|hash| Key::Block(hash.clone()).key().to_vec())
                    .collect::<Vec<_>>(),
            )?
            .into_iter()
            .filter_map(|(_, value)| codec::decode(&value).ok())
            .collect::<Vec<_>>())
    }
}

/// The key of the sync storage
pub enum Key {
    /// The finalized head.
    Finalized,

    /// The beefy root of the given block hash.
    BeefyRoot(OpaqueHash),

    /// The block of the given hash.
    Block(OpaqueHash),

    /// The descendant of the given hash in finalized chain.
    Descendant(OpaqueHash),

    /// The header of the given block hash.
    Header(OpaqueHash),

    /// The parent of the given block hash.
    Parent(OpaqueHash),

    /// The state root of the given block hash.
    StateRoot(OpaqueHash),
}

impl Key {
    /// Get the key of the sync storage
    pub fn key(&self) -> [u8; 31] {
        let mut key = [0; 31];
        match self {
            Key::Finalized => {}
            Key::BeefyRoot(hash) => {
                key[0] = 0;
                key[1..31].copy_from_slice(hash.as_ref());
            }
            Key::Block(hash) => {
                key[0] = 1;
                key[1..31].copy_from_slice(hash.as_ref());
            }
            Key::Descendant(hash) => {
                key[0] = 2;
                key[1..31].copy_from_slice(hash.as_ref());
            }
            Key::Header(hash) => {
                key[0] = 3;
                key[1..31].copy_from_slice(hash.as_ref());
            }
            Key::StateRoot(hash) => {
                key[0] = 4;
                key[1..31].copy_from_slice(hash.as_ref());
            }
            Key::Parent(hash) => {
                key[0] = 5;
                key[1..31].copy_from_slice(hash.as_ref());
            }
        }

        key
    }
}
