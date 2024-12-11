//! Shard utils

use anyhow::Result;

const SHARD_ALIGNMENT: usize = 64;
const MAX_CHUNKS: u16 = 16384;
const CHUNKS_THRESHOLD: u16 = 3;

/// Create shards from data
pub fn make(original_count: u16, data: &[u8]) -> Result<Vec<Vec<u8>>> {
    if original_count == 0 || data.is_empty() {
        return Ok(Default::default());
    }

    // Calculate the number of bytes per shard
    let size = size(original_count, data.len());
    if size == 0 {
        return Ok(Default::default());
    }

    // Create the shards
    let mut result = vec![vec![0u8; size]; original_count as usize];
    for (i, chunk) in data.chunks(size).enumerate() {
        result[i][..chunk.len()].as_mut().copy_from_slice(chunk);
    }

    Ok(result)
}

/// Obtain a threshold of chunks that should be enough to recover the data.
pub fn recoverable(chunks: u16) -> Result<(u16, u16)> {
    if chunks > MAX_CHUNKS {
        anyhow::bail!("Too many chunks");
    }
    if chunks == 0 {
        anyhow::bail!("Not enough chunks");
    }

    // for every chunk, we need at least 3 chunks to recover the data
    let needed = (chunks - 1) / CHUNKS_THRESHOLD + 1;
    Ok((needed, chunks - needed))
}

/// Calculate the number of bytes per shard
pub fn size(chunks: u16, data_len: usize) -> usize {
    let shards = data_len.div_ceil(chunks as usize);
    shards.div_ceil(SHARD_ALIGNMENT) * SHARD_ALIGNMENT
}
