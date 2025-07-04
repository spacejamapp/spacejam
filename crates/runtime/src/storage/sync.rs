//! Block storage

use crate::storage::{ArchiveStorage, Column, KVStorage};
use anyhow::Result;
use score::{
    block::{Head, Header},
    extrinsic::TicketsOrKeys,
    Block, OpaqueHash,
};

/// Sync storage
pub trait SyncStorage: KVStorage + ArchiveStorage {
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

    /// Get the prefix iterator of the sync storage
    fn sync_prefix_iter(
        &self,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        self.prefix_iter(Column::Sync, prefix)
    }

    /// Get the batch read of the sync storage
    fn sync_batch_read(&self, keys: Vec<Vec<u8>>) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        self.batch_read(Column::Sync, keys)
    }

    /// Get the block from the sync storage
    fn block(&self, hash: &OpaqueHash) -> Result<Block> {
        let key = Key::Block(*hash).key();
        let value = self
            .sync_get(key)?
            .ok_or(anyhow::anyhow!("Block not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Get the descendant of the given hash in finalized chain.
    fn descendant(&self, parent: &OpaqueHash) -> Result<OpaqueHash> {
        let key = Key::Descendant(*parent).key();
        let value = self
            .sync_get(key)?
            .ok_or(anyhow::anyhow!("Descendant not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Get the finalized head
    fn finalized(&self) -> Result<Head> {
        let key = Key::Finalized.key();
        let value = self
            .sync_get(key)?
            .ok_or(anyhow::anyhow!("Finalized head not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Set the finalized head
    ///
    /// FIXME: use a commit instead.
    fn finalize(&self, block: &Block, hash: OpaqueHash, state_root: OpaqueHash) -> Result<()> {
        self.sync_set(Key::Block(hash).key(), codec::encode(block)?)?;
        self.sync_set(Key::Descendant(block.header.parent).key(), hash)?;
        self.sync_set(Key::Finalized.key(), codec::encode(&block.header.head()?)?)?;
        self.sync_set(Key::Header(hash).key(), codec::encode(&block.header)?)?;
        self.sync_set(Key::Parent(hash).key(), block.header.parent)?;
        self.sync_set(Key::StateRoot(hash).key(), state_root)?;
        if let Some(tickets) = block.header.tickets_mark {
            let epoch = block.header.slot / score::EPOCH_LENGTH + 1;
            self.sync_set(
                Key::Safrole(epoch).key(),
                codec::encode(&TicketsOrKeys::Tickets(tickets))?,
            )?;
        }

        self.archive(&hash)?;
        Ok(())
    }

    /// Get the header
    fn header(&self, hash: &OpaqueHash) -> Result<Header> {
        let key = Key::Header(*hash).key();
        let value = self
            .sync_get(key)?
            .ok_or(anyhow::anyhow!("Header not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Get the parent
    fn parent(&self, block: &OpaqueHash) -> Result<OpaqueHash> {
        let key = Key::Parent(*block).key();
        let value = self
            .sync_get(key)?
            .ok_or(anyhow::anyhow!("Parent not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    fn series(&self, epoch: u32) -> Result<TicketsOrKeys> {
        let key = Key::Safrole(epoch).key();
        let value = self
            .sync_get(key)?
            .ok_or(anyhow::anyhow!("Series not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Get the state root
    fn state_root(&self, block: &OpaqueHash) -> Result<Option<OpaqueHash>> {
        let key = Key::StateRoot(*block).key();
        let value = self.sync_get(key)?;
        if let Some(value) = value {
            Ok(Some(codec::decode(value.as_ref())?))
        } else {
            Ok(None)
        }
    }

    /// Get the beefy root
    fn beefy_root(&self, block: &OpaqueHash) -> Result<OpaqueHash> {
        let key = Key::BeefyRoot(*block).key();
        let value = self
            .sync_get(key)?
            .ok_or(anyhow::anyhow!("Beefy root not found"))?;
        Ok(codec::decode(value.as_ref())?)
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

    /// The safrole keys
    Safrole(u32),

    /// The state root of the given block hash.
    StateRoot(OpaqueHash),
}

impl Key {
    /// Get the key of the sync storage
    pub fn key(&self) -> [u8; 31] {
        let mut key = [0; 31];
        match self {
            Key::Finalized => {
                key = [255; 31];
            }
            Key::BeefyRoot(hash) => {
                key[0] = 0;
                key[..31].copy_from_slice(&hash[..30]);
            }
            Key::Block(hash) => {
                key[0] = 1;
                key[1..].copy_from_slice(&hash[..30]);
            }
            Key::Descendant(hash) => {
                key[0] = 2;
                key[1..].copy_from_slice(&hash[..30]);
            }
            Key::Header(hash) => {
                key[0] = 3;
                key[1..].copy_from_slice(&hash[..30]);
            }
            Key::Safrole(epoch) => {
                key[0] = 4;
                key[1..5].copy_from_slice(&epoch.to_le_bytes());
            }
            Key::StateRoot(hash) => {
                key[0] = 5;
                key[1..].copy_from_slice(&hash[..30]);
            }
            Key::Parent(hash) => {
                key[0] = 6;
                key[1..].copy_from_slice(&hash[..30]);
            }
        }

        key
    }
}
