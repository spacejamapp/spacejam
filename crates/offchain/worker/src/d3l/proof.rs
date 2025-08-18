//! Protocol-specific proof structures for segment verification

use crate::d3l::justification::{Justification, JustificationPath};
use anyhow::Result;
use score::{OpaqueHash, Segment};
use serde::{Deserialize, Serialize};

/// Justification for a specific segment shard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentShardJustification {
    /// The segment index within the bundle
    pub segment_index: u16,
    /// The shard index within the segment
    pub shard_index: u16,
    /// The justification path
    pub path: JustificationPath,
}

/// Justification for a work-package bundle shard
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

/// Page-proof containing 64 segment hashes + Merkle proof per Gray Paper
/// Implements the P(segments) function from Gray Paper equation for efficient segment justification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageProof {
    /// Page of segment hashes (max 64 per Gray Paper specification)
    pub segment_hashes: Vec<OpaqueHash>,
    /// Merkle proof from segments-root to this subtree
    pub merkle_proof: Vec<Justification>,
    /// Page index within the segment set
    pub page_index: u16,
}

impl PageProof {
    /// Create new page-proof from segments and page metadata
    pub fn new(
        segment_hashes: Vec<OpaqueHash>,
        merkle_proof: Vec<Justification>,
        page_index: u16,
    ) -> Self {
        Self {
            segment_hashes,
            merkle_proof,
            page_index,
        }
    }

    /// Generate page-proof for a page of segments (Gray Paper P function implementation)
    pub fn generate(
        segment_hashes: &[OpaqueHash],
        page_index: u16,
        segments_root: &OpaqueHash,
    ) -> Result<Self> {
        if segment_hashes.is_empty() {
            return Err(anyhow::anyhow!(
                "Cannot generate page-proof for empty segments"
            ));
        }

        if segment_hashes.len() > 64 {
            // Gray Paper: 64 segments per page
            return Err(anyhow::anyhow!(
                "Page size exceeds Gray Paper limit of 64 segments"
            ));
        }

        // Generate Merkle proof from segments_root to this subtree
        // For now, create a simple proof - this will be enhanced with proper tree traversal
        let merkle_proof = vec![Justification::Hash(*segments_root)];

        Ok(PageProof {
            segment_hashes: segment_hashes.to_vec(),
            merkle_proof,
            page_index,
        })
    }

    /// Generate page-proofs for a set of exported segments (function P)
    pub async fn proofs(
        exported: &[Segment],
        exports_root: &OpaqueHash,
    ) -> Result<(Vec<Self>, Vec<OpaqueHash>)> {
        let all_segment_hashes: Vec<OpaqueHash> = exported
            .iter()
            .map(|segment| crypto::blake2b(segment))
            .collect();

        // split into pages of 64 segments
        let page_count = exported.len().div_ceil(64);
        let mut page_proofs = Vec::new();
        for page_index in 0..page_count {
            let start_idx = page_index * 64;
            let end_idx = std::cmp::min(start_idx + 64, exported.len());
            let page_segment_hashes = &all_segment_hashes[start_idx..end_idx];
            let page_proof =
                PageProof::generate(page_segment_hashes, page_index as u16, exports_root)?;
            page_proofs.push(page_proof);
        }

        Ok((page_proofs, all_segment_hashes))
    }

    /// Verify a segment using this page-proof
    pub fn verify_segment(&self, segment: &Segment, segment_index_in_page: u16) -> Result<bool> {
        if (segment_index_in_page as usize) >= self.segment_hashes.len() {
            return Ok(false);
        }

        let segment_hash = crypto::blake2b(segment);
        let expected_hash = self.segment_hashes[segment_index_in_page as usize];
        Ok(segment_hash == expected_hash)
    }

    /// Get the number of segments in this page
    pub const fn segment_count(&self) -> usize {
        self.segment_hashes.len()
    }
}
