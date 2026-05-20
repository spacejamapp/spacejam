//! Shard utilities for segment validation and reconstruction

use crate::d3l::BundleShardJustification;
use anyhow::Result;
use erasure::{Config, decode_sync, encode_sync};
use score::SEGMENT_SIZE;

/// A shard of a work report
pub struct Shard {
    /// The bundle shard
    pub bundle: Vec<u8>,

    /// The segment shards
    pub segment_shards: Vec<u8>,

    /// The bundle shard justifications
    pub justifications: BundleShardJustification,
}

/// Get minimum shards needed for reconstruction (Reed-Solomon needs original count)
///
/// TODO: use constant for this
pub fn min_shards() -> usize {
    Config::default().original
}

/// Validate that a segment produces the expected shard at given index
pub fn verify_shard(segment: &[u8], expected_shard: &[u8], shard_index: u16) -> Result<bool> {
    let shards = encode_sync(segment.to_vec())?;
    if (shard_index as usize) >= shards.len() {
        return Ok(false);
    }

    let actual_shard = &shards[shard_index as usize];
    Ok(actual_shard == expected_shard)
}

/// Reconstruct segment with size validation (generic over size)
pub fn reconstruct_segment(
    partial_shards: &[(usize, Vec<u8>)],
) -> Result<[u8; SEGMENT_SIZE]> {
    let reconstructed = decode_sync(partial_shards.to_vec())?;

    if reconstructed.len() != SEGMENT_SIZE {
        return Err(anyhow::anyhow!(
            "Invalid segment size: {} != {}",
            reconstructed.len(),
            SEGMENT_SIZE
        ));
    }

    let mut segment = [0u8; SEGMENT_SIZE];
    segment.copy_from_slice(&reconstructed);
    Ok(segment)
}

/// Build partial shards for reconstruction from available shards
pub fn partial_shards(shards: &[Vec<u8>]) -> Vec<(usize, Vec<u8>)> {
    let needed = Config::default().original;
    shards
        .iter()
        .enumerate()
        .take(needed)
        .map(|(i, shard)| (i, shard.clone()))
        .collect()
}
