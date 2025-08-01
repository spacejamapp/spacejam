//! Justification types and utilities for segment verification

use anyhow::Result;
use score::OpaqueHash;
use serde::{Deserialize, Serialize};

/// Justification for segment/bundle shard correctness per network protocol
/// Discriminators match CE 137/139/140 protocol specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Justification {
    /// Discriminator 0: Single hash (leaf node in co-path)
    Hash(OpaqueHash),
    /// Discriminator 1: Hash pair (internal node in co-path)
    HashPair(OpaqueHash, OpaqueHash),
    /// Discriminator 2: Segment shard data (for CE 140 protocol)
    SegmentShard(Vec<u8>),
}

impl Justification {
    /// Verify a shard against this justification
    pub fn verify(&self, shard: &[u8], erasure_root: &OpaqueHash) -> Result<bool> {
        let shard_hash = crypto::blake2b(shard);
        let path = JustificationPath::new(*erasure_root, 0, vec![self.clone()]);
        path.verify_shard(&shard_hash)
    }
}

/// Merkle co-path from erasure root to a specific shard
/// Used to prove shard correctness without revealing full tree
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JustificationPath {
    /// The erasure root this path proves against
    pub erasure_root: OpaqueHash,
    /// The shard index being justified
    pub shard_index: u16,
    /// The co-path elements from shard to root
    pub path: Vec<Justification>,
}

impl JustificationPath {
    /// Create a new justification path
    pub fn new(erasure_root: OpaqueHash, shard_index: u16, path: Vec<Justification>) -> Self {
        Self {
            erasure_root,
            shard_index,
            path,
        }
    }

    /// Compute justification path from shards for a specific shard index
    pub fn compute(
        erasure_root: &OpaqueHash,
        shard_index: u16,
        shards: &[Vec<u8>],
    ) -> Result<Option<Self>> {
        let merkle_tree = crypto::merkle::MerkleTree::from(shards.to_vec());
        if merkle_tree.root() != *erasure_root || (shard_index as usize) >= shards.len() {
            return Ok(None);
        }

        let shard_hash = crypto::blake2b(&shards[shard_index as usize]);
        let Some(proof) = merkle_tree.proof(shard_hash) else {
            return Ok(None);
        };

        let path = JustificationPath::from_merkle_proof(*erasure_root, shard_index, &proof.proof);
        Ok(Some(path))
    }

    /// Verify that a shard matches this justification path
    pub fn verify_shard(&self, shard_hash: &OpaqueHash) -> Result<bool> {
        if self.path.is_empty() {
            return Ok(*shard_hash == self.erasure_root);
        }

        let mut current_hash = *shard_hash;
        let mut index = self.shard_index as usize;
        for justification in &self.path {
            match justification {
                Justification::Hash(sibling_hash) => {
                    current_hash = if index % 2 == 0 {
                        crypto::blake2b(&[&current_hash[..], &sibling_hash[..]].concat())
                    } else {
                        crypto::blake2b(&[&sibling_hash[..], &current_hash[..]].concat())
                    };
                }
                Justification::HashPair(left, right) => {
                    current_hash = crypto::blake2b(&[&left[..], &right[..]].concat());
                }
                Justification::SegmentShard(_) => {
                    return Err(anyhow::anyhow!(
                        "SegmentShard justification not supported in verification path"
                    ));
                }
            }
            index /= 2;
        }

        Ok(current_hash == self.erasure_root)
    }

    /// Build justification path from Merkle proof
    pub fn from_merkle_proof(
        erasure_root: OpaqueHash,
        shard_index: u16,
        proof_path: &[OpaqueHash],
    ) -> Self {
        let path = proof_path
            .iter()
            .map(|&hash| Justification::Hash(hash))
            .collect();

        Self::new(erasure_root, shard_index, path)
    }
}

/// Justification for a specific segment shard (CE 139/140 protocols)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentShardJustification {
    /// The segment index within the bundle
    pub segment_index: u16,
    /// The shard index within the segment
    pub shard_index: u16,
    /// The justification path
    pub path: JustificationPath,
}

/// Justification for a work-package bundle shard (CE 137/138 protocols)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleShardJustification {
    /// The shard index within the bundle
    pub shard_index: u16,
    /// The justification path
    pub path: JustificationPath,
}

impl BundleShardJustification {
    /// Create new bundle shard justification from shards
    pub fn new(
        shards: &[Vec<u8>],
        erasure_root: &OpaqueHash,
        shard_index: u16,
    ) -> Result<Option<Self>> {
        let Some(path) = JustificationPath::compute(erasure_root, shard_index, shards)? else {
            return Ok(None);
        };

        Ok(Some(Self { shard_index, path }))
    }

    /// Verify bundle shard against justification
    pub fn verify_bundle_shard(&self, shard: &[u8]) -> Result<bool> {
        let shard_hash = crypto::blake2b(shard);
        self.path.verify_shard(&shard_hash)
    }
}

impl SegmentShardJustification {
    /// Create new segment shard justification from shards
    pub fn new(
        shards: &[Vec<u8>],
        erasure_root: &OpaqueHash,
        segment_index: u16,
        shard_index: u16,
    ) -> Result<Option<Self>> {
        let Some(path) = JustificationPath::compute(erasure_root, shard_index, shards)? else {
            return Ok(None);
        };

        Ok(Some(Self {
            segment_index,
            shard_index,
            path,
        }))
    }

    /// Verify segment shard against justification
    pub fn verify_segment_shard(&self, shard: &[u8]) -> Result<bool> {
        let shard_hash = crypto::blake2b(shard);
        self.path.verify_shard(&shard_hash)
    }
}
