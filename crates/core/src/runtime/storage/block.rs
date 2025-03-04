//! Block storage

use crate::{
    runtime::{storage::KVStorage, Head},
    Block, OpaqueHash, TimeSlot,
};
use anyhow::Result;

const PREFIX: &[u8] = b"block";
const FINALIZED_KEY: &[u8] = b"finalized";
const BLOCK_HASH_KEY: &[u8] = b"block_hash";
const BLOCK_SLOT_KEY: &[u8] = b"block_slot";

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

    /// Get the finalized head
    fn get_finalized(&self) -> Result<Head> {
        self.get(FINALIZED_KEY)?
            .ok_or_else(|| anyhow::anyhow!("Finalized head not found"))
            .and_then(|value| Ok(codec::decode(&value)?))
    }

    /// Set the finalized head
    fn set_finalized(&self, head: &Head) -> Result<()> {
        self.set(FINALIZED_KEY, &codec::encode(head)?)?;

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

    /// Get the block hash by slot
    fn get_hash(&self, slot: TimeSlot) -> Result<OpaqueHash> {
        let key = [BLOCK_HASH_KEY, &slot.to_le_bytes()].concat();
        self.get(&key)?
            .ok_or(anyhow::anyhow!("Block hash not found"))
            .and_then(|value| Ok(codec::decode(&value)?))
    }

    /// Get the block slot by hash
    fn get_slot(&self, hash: &OpaqueHash) -> Result<TimeSlot> {
        let key = [BLOCK_SLOT_KEY, hash.as_ref()].concat();
        self.get(&key)?
            .ok_or(anyhow::anyhow!("Block slot not found"))
            .and_then(|value| Ok(codec::decode(&value)?))
    }
}
