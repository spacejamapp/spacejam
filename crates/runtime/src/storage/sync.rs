//! Block storage

use crate::Storage;
use anyhow::Result;
use score::{
    Block, OpaqueHash,
    block::Head,
    extrinsic::{TicketBody, TicketsOrKeys},
    safrole::ValidatorIter,
};

/// The key for the block storage
pub const BLOCK_KEY: &[u8] = b"block";

/// The key for the series storage
pub const SERIES_KEY: &[u8] = b"series";

/// Sync storage
pub trait SyncStorage: Storage {
    /// Get the block by hash
    fn get_block(&self, hash: &OpaqueHash) -> Result<Block> {
        let key = [BLOCK_KEY, hash.as_ref()].concat();
        let value = self.get(&key)?.ok_or(anyhow::anyhow!("Block not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Save the block
    fn set_block(&self, block: &Block) -> Result<()> {
        let hash = block.header.hash()?;
        let key = [BLOCK_KEY, hash.as_ref()].concat();
        self.set(&key, &codec::encode(block)?)?;
        Ok(())
    }

    /// Set the best head
    fn set_best(&self, head: &Head) -> Result<()> {
        let key = [BLOCK_KEY, b"best"].concat();
        self.set(&key, &codec::encode(head)?)?;
        Ok(())
    }

    /// Get the best head
    fn get_best(&self) -> Result<Head> {
        let key = [BLOCK_KEY, b"best"].concat();
        let value = self
            .get(&key)?
            .ok_or(anyhow::anyhow!("Best head not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Set the finalized head
    fn set_finalized(&self, head: &Head) -> Result<()> {
        let key = [BLOCK_KEY, b"finalized"].concat();
        self.set(&key, &codec::encode(head)?)?;
        Ok(())
    }

    /// Get the finalized head
    fn get_finalized(&self) -> Result<Head> {
        let key = [BLOCK_KEY, b"finalized"].concat();
        let value = self
            .get(&key)?
            .ok_or(anyhow::anyhow!("Finalized head not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Set the parent
    fn set_parent(&self, block: &OpaqueHash, parent: &Head) -> Result<()> {
        let mut key = [BLOCK_KEY, b"parent"].concat();
        key.extend_from_slice(block.as_ref());
        self.set(&key, &codec::encode(parent)?)?;
        Ok(())
    }

    /// Get the parent
    fn get_parent(&self, block: &OpaqueHash) -> Result<Option<Head>> {
        let mut key = [BLOCK_KEY, b"parent"].concat();
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
        let mut key = [BLOCK_KEY, b"state_root"].concat();
        key.extend_from_slice(block.as_ref());
        self.set(&key, &codec::encode(root)?)?;
        Ok(())
    }

    /// Get the state root
    fn get_state_root(&self, block: &OpaqueHash) -> Result<Option<OpaqueHash>> {
        let mut key = [BLOCK_KEY, b"state_root"].concat();
        key.extend_from_slice(block.as_ref());
        let value = self.get(&key)?;
        if let Some(value) = value {
            Ok(Some(codec::decode(value.as_ref())?))
        } else {
            Ok(None)
        }
    }

    /// Set the beefy root
    fn set_beefy_root(&self, block: &OpaqueHash, root: &OpaqueHash) -> Result<()> {
        let mut key = [BLOCK_KEY, b"beefy_root"].concat();
        key.extend_from_slice(block.as_ref());
        self.set(&key, &codec::encode(root)?)?;
        Ok(())
    }

    /// Get the beefy root
    fn get_beefy_root(&self, block: &OpaqueHash) -> Result<OpaqueHash> {
        let mut key = [BLOCK_KEY, b"beefy_root"].concat();
        key.extend_from_slice(block.as_ref());
        let value = self
            .get(&key)?
            .ok_or(anyhow::anyhow!("Beefy root not found"))?;
        Ok(codec::decode(value.as_ref())?)
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

    /// Get the next series
    fn next_series(&self) -> Result<[TicketBody; score::EPOCH_LENGTH as usize]> {
        let key = [SERIES_KEY, b"next"].concat();
        let value = self.get(&key)?.ok_or(anyhow::anyhow!("Series not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Set the next series
    fn set_next_series(&self, series: [TicketBody; score::EPOCH_LENGTH as usize]) -> Result<()> {
        let key = [SERIES_KEY, b"next"].concat();
        self.set(&key, &codec::encode(&series)?)?;
        Ok(())
    }

    /// Fetch the series
    fn series(&self) -> Result<TicketsOrKeys> {
        if let Ok(Some(value)) = self.get(SERIES_KEY) {
            codec::decode(value.as_ref()).map_err(Into::into)
        } else {
            Ok(self.safrole()?.series)
        }
    }

    /// On new epoch handler for rotating the series
    fn on_new_epoch(&self) -> Result<()> {
        if let Ok(series) = self.next_series() {
            self.set(SERIES_KEY, codec::encode(&TicketsOrKeys::Tickets(series))?)?;
        } else {
            let keys = self.next_validators()?.bandersnatch();
            let entropy = self.entropy()?;
            let series = TicketsOrKeys::fallback(keys, entropy[1]);
            self.set(SERIES_KEY, codec::encode(&series)?)?;
        }

        Ok(())
    }
}
