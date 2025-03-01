//! Block storage

use crate::{runtime::storage::KVStorage, Block, OpaqueHash};
use anyhow::Result;

const PREFIX: &[u8] = b"block";
const FINALIZED_KEY: &[u8] = b"finalized";

/// Block storage
pub trait BlockStorage: KVStorage {
    /// Get the block by hash
    fn get_block(&self, hash: &OpaqueHash) -> Result<Block> {
        let key = [PREFIX, hash.as_ref()].concat();
        let value = self.get(&key)?.ok_or(anyhow::anyhow!("Block not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Save the block
    fn save_block(&self, block: &Block) -> Result<()> {
        let hash = block.header.hash()?;
        let key = [PREFIX, hash.as_ref()].concat();
        self.set(&key, &codec::encode(block)?)?;
        Ok(())
    }

    /// Drop the blocks
    fn drop_blocks(&self, hashes: &[OpaqueHash]) -> Result<()> {
        for hash in hashes {
            let key = [PREFIX, hash.as_ref()].concat();
            self.remove(&key)?;
        }
        Ok(())
    }

    /// Get the hash of the latest finalized block
    ///
    /// Returns None if no block has been finalized yet
    fn get_finalized_block_hash(&self) -> Result<Option<OpaqueHash>> {
        match self.get(FINALIZED_KEY)? {
            Some(value) => Ok(Some(codec::decode(&value)?)),
            None => Ok(None),
        }
    }

    /// Set the hash of the latest finalized block
    fn set_finalized_block_hash(&self, hash: &OpaqueHash) -> Result<()> {
        self.set(FINALIZED_KEY, &codec::encode(hash)?)?;
        Ok(())
    }

    /// Check if a block exists in storage
    fn block_exists(&self, hash: &OpaqueHash) -> Result<bool> {
        let key = [PREFIX, hash.as_ref()].concat();
        Ok(self.get(&key)?.is_some())
    }
}
