//! Block storage

use crate::{
    runtime::{storage::KVStorage, Head},
    Block, OpaqueHash,
};
use anyhow::Result;

const BLOCK_KEY: &[u8] = b"block";
const FINALIZED_KEY: &[u8] = b"finalized";
const BLOCK_HASH_KEY: &[u8] = b"block_hash";
const BLOCK_SLOT_KEY: &[u8] = b"block_slot";

/// Block storage
pub trait BlockStorage: KVStorage {
    /// Get the block by hash
    fn get_block(&self, hash: &OpaqueHash) -> Result<Block> {
        let key = [BLOCK_KEY, hash.as_ref()].concat();
        let value = self.get(&key)?.ok_or(anyhow::anyhow!("Block not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Save the block
    fn save_block(&self, block: &Block) -> Result<()> {
        let hash = block.header.hash()?;
        let key = [BLOCK_KEY, hash.as_ref()].concat();
        self.set(&key, &codec::encode(block)?)?;

        // Save the head
        self.save_head(&Head {
            hash,
            slot: block.header.slot,
        })?;
        Ok(())
    }

    fn save_head(&self, head: &Head) -> Result<()> {
        // set block hash indexed by slot
        {
            let key = [BLOCK_HASH_KEY, &head.slot.to_le_bytes()].concat();
            self.set(&key, &codec::encode(&head.hash)?)?;
        }

        // set block slot indexed by hash
        {
            let key = [BLOCK_SLOT_KEY, head.hash.as_ref()].concat();
            self.set(&key, &codec::encode(&head.slot)?)?;
        }
        Ok(())
    }

    /// Fetch the blocks
    fn fetch_blocks(&self, hashes: &[OpaqueHash]) -> Result<Vec<Block>> {
        Ok(self
            .batch_read(
                hashes
                    .iter()
                    .map(|hash| [BLOCK_KEY, hash.as_ref()].concat())
                    .collect::<Vec<_>>(),
            )?
            .into_iter()
            .filter_map(|(_, value)| codec::decode(&value).ok())
            .collect::<Vec<_>>())
    }

    /// Get the finalized head
    fn get_finalized(&self) -> Result<Head> {
        self.get(FINALIZED_KEY)?
            .ok_or_else(|| anyhow::anyhow!("Finalized head not found"))
            .and_then(|value| Ok(codec::decode(&value)?))
    }

    /// Set the finalized head
    fn set_finalized(&self, head: &Head) -> Result<()> {
        self.set(FINALIZED_KEY, &codec::encode(head)?)?;
        self.save_head(head)?;
        Ok(())
    }
}
