//! Erasure encoding and decoding

use anyhow::Result;

pub mod shard;

/// Encode data into erasure-coded chunks.
pub fn encode(chunks: u16, data: &[u8]) -> Result<Vec<Vec<u8>>> {
    let (count, recovery_count) = shard::recoverable(chunks)?;
    let mut shards = shard::make(count, data)?;
    let recovery = reed_solomon::encode(count as usize, recovery_count as usize, shards.iter())?;

    shards.extend(recovery);
    Ok(shards)
}

/// Decode erasure-coded chunks.
pub fn decode<B, I>(chunks: u16, len: usize, shards: I) -> Result<Vec<u8>>
where
    I: IntoIterator<Item = B>,
    B: AsRef<[u8]> + Clone,
{
    if chunks == 1 {
        return shards
            .into_iter()
            .next()
            .map(|v| v.as_ref().to_vec())
            .ok_or(anyhow::anyhow!("Not enough shards"));
    }

    // construct the original and recovery shards
    let (count, recovery_count) = shard::recoverable(chunks)?;
    let (mut original, recovery): (Vec<_>, Vec<_>) = shards
        .into_iter()
        .enumerate()
        .partition(|(i, _)| *i < count as usize);

    original.sort_by_key(|(i, _)| *i);
    let recovery = recovery.into_iter().map(|(i, v)| (i - count as usize, v));

    // decode the shards
    let (mb_index, mb_bytes) = original[0].clone();
    let mut recovered = reed_solomon::decode(
        count as usize,
        recovery_count as usize,
        original.into_iter(),
        recovery,
    )?;

    // reconstruct the data
    let size = shard::size(chunks, len);
    let mut data = vec![0; size * count as usize];
    for i in 0..count as usize {
        if let Some(chunk) = recovered.remove(&i) {
            data.extend_from_slice(chunk.as_ref());
        } else {
            if mb_index != i {
                anyhow::bail!("index mismatch");
            }
            data.extend_from_slice(mb_bytes.as_ref());
        }
    }

    data.truncate(len);
    Ok(data)
}
