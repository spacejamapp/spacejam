//! Shard utilities for segment validation and reconstruction

use anyhow::Result;
use erasure::{decode_sync, encode_sync, Config};
use score::SEGMENT_SIZE;

use crate::d3l::BundleShardJustification;

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

/// Validate that shards form a valid Merkle tree with expected root
pub fn verify_root(shards: &[Vec<u8>], expected_root: &[u8; 32]) -> Result<bool> {
    let merkle_tree = crypto::merkle::MerkleTree::from(shards.to_vec());
    Ok(merkle_tree.root() == *expected_root)
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
) -> Result<[u8; SEGMENT_SIZE as usize]> {
    let reconstructed = decode_sync(partial_shards.to_vec())?;

    if reconstructed.len() != SEGMENT_SIZE as usize {
        return Err(anyhow::anyhow!(
            "Invalid segment size: {} != {}",
            reconstructed.len(),
            SEGMENT_SIZE
        ));
    }

    let mut segment = [0u8; SEGMENT_SIZE as usize];
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

/// Validate segment reconstruction pipeline (generic over size)
pub fn validate_reconstruction(
    shards: &[Vec<u8>],
    shard_index: u16,
    expected_shard: &[u8],
) -> Result<[u8; SEGMENT_SIZE as usize]> {
    let partial = partial_shards(shards);
    let segment = reconstruct_segment(&partial)?;

    // Validate reconstructed segment produces the expected shard
    if !verify_shard(&segment, expected_shard, shard_index)? {
        return Err(anyhow::anyhow!(
            "Reconstructed segment validation failed for shard {}",
            shard_index
        ));
    }

    Ok(segment)
}
