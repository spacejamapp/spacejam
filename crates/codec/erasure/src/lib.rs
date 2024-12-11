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

    // decode the shards
    let recovery = recovery.into_iter().map(|(i, v)| (i - count as usize, v));
    let mut recovered = reed_solomon::decode(
        count as usize,
        recovery_count as usize,
        original.iter().map(|(i, v)| (*i, v.as_ref())),
        recovery,
    )?;

    // reconstruct the data
    let size = shard::size(chunks, len);
    let mut data = Vec::with_capacity(size * count as usize);
    let mut original = original.into_iter();
    for i in 0..count as usize {
        if let Some(chunk) = recovered.remove(&i) {
            data.extend_from_slice(chunk.as_ref());
        } else {
            let (index, value) = original
                .next()
                .ok_or(anyhow::anyhow!("not enough shards"))?;

            if index != i {
                anyhow::bail!("index mismatch, should be {i}, got {index}");
            }

            data.extend_from_slice(value.as_ref());
        }
    }

    data.truncate(len);
    Ok(data)
}

#[test]
fn hello_world_coding() -> Result<()> {
    let data = b"hello world".to_vec();
    let size = 4;
    let chunks = encode(size, &data)?;
    let decoded = decode(size, data.len(), chunks)?;
    assert_eq!(data, decoded, "data mismatch");
    Ok(())
}

#[test]
fn large_data_coding() -> Result<()> {
    let data = (0..u16::MAX).map(|i| i as u8).collect::<Vec<_>>();
    let size = 4;
    let chunks = encode(size, &data)?;
    let decoded = decode(size, data.len(), chunks)?;
    assert_eq!(data, decoded, "data mismatch");
    Ok(())
}
