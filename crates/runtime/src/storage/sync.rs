//! Block storage

use crate::Storage;
use anyhow::Result;
use score::{
    Block, OpaqueHash,
    extrinsic::{TicketBody, TicketsOrKeys},
};

const BLOCK_KEY: &[u8] = b"block";
const SERIES_KEY: &[u8] = b"series";

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

    /// On new epoch handler for rotating the series
    fn on_new_epoch(&self) -> Result<()> {
        let next_series = [SERIES_KEY, b"next"].concat();
        if let Some(value) = self.get(&next_series)? {
            let series: Vec<TicketBody> = codec::decode(&value)?;
            self.set(SERIES_KEY, codec::encode(&TicketsOrKeys::Tickets(series))?)?;
        } else {
            self.remove(SERIES_KEY)?;
        }

        Ok(())
    }

    /// Get the next series
    fn next_series(&self) -> Result<Vec<TicketBody>> {
        let key = [SERIES_KEY, b"next"].concat();
        let value = self.get(&key)?.ok_or(anyhow::anyhow!("Series not found"))?;
        Ok(codec::decode(value.as_ref())?)
    }

    /// Set the next series
    fn set_next_series(&self, series: &[TicketBody]) -> Result<()> {
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
}
