//! Block storage

use crate::Storage;
use anyhow::Result;
use score::{
    block::{Head, Header},
    extrinsic::TicketsOrKeys,
    safrole::ValidatorIter,
    Block, OpaqueHash,
};

/// The key for the sync storage
pub const SYNC: &[u8] = b"sync";

/// Sync storage
pub trait SyncStorage: Storage {
    /// Get the ancestors of the given hash.
    fn ancestors(&self, hash: &OpaqueHash, ancestor: &OpaqueHash) -> Result<Vec<OpaqueHash>> {
        let mut ancestors = Vec::new();
        let mut current = *hash;
        while let Some(parent) = self.parent(&current)? {
            current.copy_from_slice(parent.as_ref());
            if current == *ancestor {
                break;
            }

            ancestors.push(parent);
        }

        Ok(ancestors)
    }

    /// Check if the given hash is a descendant of the ancestor.
    fn is_descendant_of(&self, hash: &OpaqueHash, ancestor: &OpaqueHash) -> bool {
        let mut key = [SYNC, b"descendant", ancestor.as_ref()].concat();
        while let Ok(Some(value)) = self.get(&key) {
            if value == hash {
                return true;
            }

            key = [SYNC, b"descendant", value.as_ref()].concat();
        }

        false
    }

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
        self.set_header(&block.header)?;
        self.set(key, codec::encode(block)?)?;
        Ok(())
    }

    /// Get the best head
    fn best(&self) -> Result<Head> {
        let key = [SYNC, b"best"].concat();
        let value = self
            .get(&key)?
            .ok_or(anyhow::anyhow!("Best head not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Set the best head
    fn set_best(&self, head: &Head) -> Result<()> {
        let key = [SYNC, b"best"].concat();
        self.set(key, codec::encode(head)?)?;
        Ok(())
    }

    /// Get the descendant of the given hash.
    fn descendant(&self, hash: &OpaqueHash) -> Result<OpaqueHash> {
        let key = [SYNC, b"descendant", hash.as_ref()].concat();
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
    fn set_finalized(&self, head: &Head) -> Result<()> {
        let key = [SYNC, b"finalized"].concat();
        self.set(key, codec::encode(head)?)?;
        Ok(())
    }

    /// Get the header
    fn header(&self, hash: &OpaqueHash) -> Result<Header> {
        let key = [SYNC, b"header", hash.as_ref()].concat();
        let value = self.get(&key)?.ok_or(anyhow::anyhow!("Header not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Set a new header to the storage
    fn set_header(&self, header: &Header) -> Result<()> {
        let hash = header.hash()?;
        let parent = header.parent;

        // set the header
        {
            let key = [SYNC, b"header", hash.as_ref()].concat();
            self.set(key, codec::encode(header)?)?;
        }

        // set the parent of this header
        {
            let key = [SYNC, b"parent", hash.as_ref()].concat();
            self.set(key, parent)?;
        }

        // set child
        //
        // FIXME: handle fork blocks
        {
            let key = [SYNC, b"descendant", parent.as_ref()].concat();
            self.set(key, hash)?;
        }
        Ok(())
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

    /// Get the series
    fn series(&self, epoch: u32) -> Result<TicketsOrKeys> {
        let key = [SYNC, b"series", epoch.to_le_bytes().as_ref()].concat();
        let value = self.get(&key)?.ok_or(anyhow::anyhow!("Series not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Set the series
    fn set_series(&self, epoch: u32, series: &TicketsOrKeys) -> Result<()> {
        let key = [SYNC, b"series", epoch.to_le_bytes().as_ref()].concat();
        self.set(key, codec::encode(&series)?)?;
        Ok(())
    }

    /// On new epoch handler for rotating the series
    ///
    /// Set the fallback series if it is not tracked.
    fn on_new_epoch(&self, epoch: u32) -> Result<()> {
        let key = [SYNC, b"series", epoch.to_le_bytes().as_ref()].concat();
        if self.series(epoch).is_ok() {
            return Ok(());
        }

        // check if the block with epoch mark left on fork chain
        //
        // FIXME: this may not correct since different nodes may have different tickets.
        let safrole = self.safrole()?;
        if safrole.accumulator.len() == score::EPOCH_LENGTH as usize {
            let mut tickets = [Default::default(); score::EPOCH_LENGTH as usize];
            tickets.copy_from_slice(&safrole.accumulator);
            self.set(key, codec::encode(&TicketsOrKeys::Tickets(tickets))?)?;
            return Ok(());
        }

        // using fallback series
        let keys = self.next_validators()?.bandersnatch();
        let entropy = self.entropy()?;
        let series = TicketsOrKeys::fallback(keys, entropy[1]);
        self.set(key, codec::encode(&series)?)?;
        Ok(())
    }
}
