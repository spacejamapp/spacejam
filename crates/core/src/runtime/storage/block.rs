//! Block storage

use crate::{runtime::storage::KVStorage, Block, OpaqueHash};
use anyhow::Result;

const PREFIX: &[u8] = b"block";

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
}
