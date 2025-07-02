//! Block storage

use anyhow::Result;
use score::{
    block::{Head, Header},
    Block, OpaqueHash,
};

use crate::storage::KVStorage;

/// The key for the sync storage
pub const SYNC: &[u8] = b"sync";

/// Sync storage
pub trait SyncStorage: KVStorage {
    /// Get the block by hash
    fn block(&self, hash: &OpaqueHash) -> Result<Block> {
        let key = [SYNC, hash.as_ref()].concat();
        let value = self.get(&key)?.ok_or(anyhow::anyhow!("Block not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Save the block
    fn set_block(&self, block: &Block) -> Result<()> {
        let hash = block.header.hash()?;
        let key = [SYNC, hash.as_ref()].concat();
        self.set(key, codec::encode(block)?)?;
        Ok(())
    }

    /// Get the descendant of the given hash in finalized chain.
    fn descendant(&self, parent: &OpaqueHash) -> Result<OpaqueHash> {
        let key = [SYNC, b"descendant", parent.as_ref()].concat();
        let value = self
            .get(&key)?
            .ok_or(anyhow::anyhow!("Descendant not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Get the finalized head
    fn finalized(&self) -> Result<Head> {
        let key = [SYNC, b"finalized"].concat();
        let value = self
            .get(&key)?
            .ok_or(anyhow::anyhow!("Finalized head not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Set the finalized head
    fn finalize(&self, head: &Head) -> Result<()> {
        let key = [SYNC, b"finalized"].concat();
        self.set(key, codec::encode(head)?)?;

        // set the descendant of the parent
        let parent = self
            .parent(&head.hash)?
            .ok_or(anyhow::anyhow!("Parent not found"))?;
        let key = [SYNC, b"descendant", parent.as_ref()].concat();
        self.set(key, head.hash)?;
        Ok(())
    }

    /// Get the header
    fn header(&self, hash: &OpaqueHash) -> Result<Header> {
        let key = [SYNC, b"header", hash.as_ref()].concat();
        let value = self.get(&key)?.ok_or(anyhow::anyhow!("Header not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Get the parent
    fn parent(&self, block: &OpaqueHash) -> Result<Option<OpaqueHash>> {
        let key = [SYNC, b"parent", block.as_ref()].concat();
        let value = self.get(&key)?;
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
        let value = self.get(&key)?;
        if let Some(value) = value {
            Ok(Some(codec::decode(value.as_ref())?))
        } else {
            Ok(None)
        }
    }

    /// Set the state root
    fn set_state_root(&self, block: &OpaqueHash, root: &OpaqueHash) -> Result<()> {
        let mut key = [SYNC, b"state_root"].concat();
        key.extend_from_slice(block.as_ref());
        self.set(key, codec::encode(root)?)?;
        Ok(())
    }

    /// Get the beefy root
    fn beefy_root(&self, block: &OpaqueHash) -> Result<OpaqueHash> {
        let mut key = [SYNC, b"beefy_root"].concat();
        key.extend_from_slice(block.as_ref());
        let value = self
            .get(&key)?
            .ok_or(anyhow::anyhow!("Beefy root not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Set the beefy root
    fn set_beefy_root(&self, block: &OpaqueHash, root: &OpaqueHash) -> Result<()> {
        let mut key = [SYNC, b"beefy_root"].concat();
        key.extend_from_slice(block.as_ref());
        self.set(key, codec::encode(root)?)?;
        Ok(())
    }

    /// Fetch the blocks
    fn fetch_blocks(&self, hashes: &[OpaqueHash]) -> Result<Vec<Block>> {
        Ok(self
            .batch_read(
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
