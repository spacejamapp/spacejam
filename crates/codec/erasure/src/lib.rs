//! Erasure encoding and decoding

use anyhow::Result;

pub mod shard;

/// Construct erasure-coded chunks.
pub fn encode(chunks: u16, data: &[u8]) -> Result<Vec<Vec<u8>>> {
    let count = shard::recoverable(chunks)?;
    let mut shards = shard::make(count, data)?;
    let recovery = reed_solomon::encode(count as usize, (chunks - count) as usize, shards.iter())?;

    shards.extend(recovery);
    Ok(shards)
}
