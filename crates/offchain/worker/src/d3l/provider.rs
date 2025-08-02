//! Segment provider trait and implementations

use crate::d3l::{shard, BundleShardJustification, PageProof, SegmentShardJustification};
use anyhow::Result;
use score::{OpaqueHash, Segment, WorkPackageHash};
use std::collections::{BTreeMap, HashMap};
use tokio::sync::RwLock;

/// Trait for data lake operations abstraction
#[allow(async_fn_in_trait)]
pub trait DataLake: Send + Sync {
    /// Retrieve page-proof for segment justification
    async fn page_proof(
        &self,
        segments_root: &OpaqueHash,
        page_index: u16,
    ) -> Result<Option<PageProof>>;

    /// Get a segment
    async fn segment(&self, segment_hash: &OpaqueHash) -> Result<Option<Segment>>;

    /// Get shards
    async fn shards(&self, segment_hash: &OpaqueHash) -> Result<Option<Vec<Vec<u8>>>>;

    /// Store page-proof for efficient segment justification (Gray Paper P function)
    async fn set_page_proof(
        &self,
        segments_root: &OpaqueHash,
        page_index: u16,
        page_proof: PageProof,
    ) -> Result<()>;

    /// set a segment
    async fn set_segment(&self, segment_hash: OpaqueHash, segment: Segment) -> Result<()>;

    /// set shards
    async fn set_shards(&self, segment_hash: OpaqueHash, shards: Vec<Vec<u8>>) -> Result<()>;

    /// Get the segment root for a work package
    async fn segment_root(&self, work_package_hash: &WorkPackageHash)
        -> Result<Option<OpaqueHash>>;

    /// Register a work package with a segment root
    async fn register_work_package(
        &self,
        work_package_hash: WorkPackageHash,
        segment_root: OpaqueHash,
    ) -> Result<()>;

    /// Check if segments are available
    async fn segments_available(&self, segment_hashes: &[OpaqueHash]) -> Result<Vec<bool>> {
        let mut availability = Vec::with_capacity(segment_hashes.len());

        for &hash in segment_hashes {
            // Short-circuit: if we have segment directly, no need to check shards
            let available = self.segment(&hash).await?.is_some()
                || (self
                    .shards(&hash)
                    .await?
                    .is_some_and(|shards| shards.len() >= shard::min_shards()));
            availability.push(available);
        }

        Ok(availability)
    }

    /// Export segments with automatic Gray Paper page-proof generation
    /// Returns (segments_root, segment_chunk_hashes) for efficient specifier creation
    async fn export_segments(
        &self,
        segments: &[Segment],
        work_package_hash: &OpaqueHash,
    ) -> Result<(OpaqueHash, Vec<OpaqueHash>)> {
        if segments.is_empty() {
            return Ok(([0u8; 32], Vec::new()));
        }

        // 1. Store individual segments and generate shards (existing logic)
        let mut all_hashes = Vec::with_capacity(segments.len() * 32);
        for segment in segments {
            let segment_hash = crypto::blake2b(segment);
            let shards = erasure::encode(segment.to_vec()).await?;

            // Store both segment and shards concurrently if possible
            self.set_segment(segment_hash, *segment).await?;
            self.set_shards(segment_hash, shards).await?;
            all_hashes.extend_from_slice(&segment_hash);
        }

        let segments_root = crypto::blake2b(&all_hashes);

        // 2. Generate and store page-proofs (Gray Paper P function)
        // Gray Paper specifies 64 segments per page for page-proofs
        let page_count = segments.len().div_ceil(64);
        let mut page_proofs = Vec::new();
        for page_index in 0..page_count {
            let start_idx = page_index * 64;
            let end_idx = std::cmp::min(start_idx + 64, segments.len());
            let page_segments = &segments[start_idx..end_idx];
            let page_proof = PageProof::generate(page_segments, page_index as u16, &segments_root)?;
            self.set_page_proof(&segments_root, page_index as u16, page_proof.clone())
                .await?;
            page_proofs.push(page_proof);
        }

        // 3. Generate segment chunks (s♣) for availability specifier
        // Combine segments and page-proofs, then erasure encode for chunk hashes
        let mut segments_with_proofs = Vec::new();
        for segment in segments {
            segments_with_proofs.extend_from_slice(segment);
        }
        for proof in &page_proofs {
            let encoded_proof = codec::encode(proof)?;
            segments_with_proofs.extend_from_slice(&encoded_proof);
        }

        let segment_chunks = if !segments_with_proofs.is_empty() {
            erasure::encode(segments_with_proofs).await?
        } else {
            Vec::new()
        };

        let segment_chunk_hashes: Vec<OpaqueHash> = segment_chunks
            .iter()
            .map(|chunk| crypto::blake2b(chunk))
            .collect();

        // 4. Register work package mapping (existing)
        self.register_work_package(*work_package_hash, segments_root)
            .await?;

        Ok((segments_root, segment_chunk_hashes))
    }

    /// Import segments
    async fn import_segments(&self, segment_hashes: &[OpaqueHash]) -> Result<Vec<Segment>> {
        let mut segments = Vec::with_capacity(segment_hashes.len());
        for &hash in segment_hashes {
            let segment = match (self.segment(&hash).await?, self.shards(&hash).await?) {
                (Some(segment), _) => segment,
                (None, Some(shards)) => {
                    let partial = shard::partial_shards(&shards);
                    shard::reconstruct_segment(&partial)?
                }
                (None, None) => return Err(anyhow::anyhow!("Segment not available: {:?}", hash)),
            };
            segments.push(segment);
        }

        Ok(segments)
    }

    /// Import segments with validation against erasure root
    async fn vimport_segments(
        &self,
        erasure_root: &OpaqueHash,
        segment_specs: &[(u16, u16)],
    ) -> Result<Vec<Segment>> {
        let Some(shards) = self.shards(erasure_root).await? else {
            return Err(anyhow::anyhow!("No shards available: {:?}", erasure_root));
        };

        if !shard::verify_root(&shards, erasure_root)? {
            return Err(anyhow::anyhow!("Invalid erasure root: {:?}", erasure_root));
        }

        let cached_segment = self.segment(erasure_root).await?;
        segment_specs
            .iter()
            .map(|&(segment_index, shard_index)| {
                let expected_shard = shards
                    .get(shard_index as usize)
                    .ok_or_else(|| anyhow::anyhow!("Shard index {} out of bounds", shard_index))?;

                match cached_segment {
                    Some(segment)
                        if shard::verify_shard(&segment, expected_shard, shard_index)? =>
                    {
                        Ok(segment)
                    }
                    Some(_) => Err(anyhow::anyhow!(
                        "Segment validation failed: {} shard {}",
                        segment_index,
                        shard_index
                    )),
                    None => shard::validate_reconstruction(&shards, shard_index, expected_shard),
                }
            })
            .collect()
    }

    /// Get the justification for a segment shard
    async fn segment_justification(
        &self,
        erasure_root: &OpaqueHash,
        segment_index: u16,
        shard_index: u16,
    ) -> Result<Option<SegmentShardJustification>> {
        let Some(shards) = self.shards(erasure_root).await? else {
            return Ok(None);
        };

        SegmentShardJustification::new(&shards, erasure_root, segment_index, shard_index)
    }

    /// Get the justification for a bundle shard
    async fn bundle_justification(
        &self,
        erasure_root: &OpaqueHash,
        shard_index: u16,
    ) -> Result<Option<BundleShardJustification>> {
        let Some(shards) = self.shards(erasure_root).await? else {
            return Ok(None);
        };
        BundleShardJustification::new(&shards, erasure_root, shard_index)
    }

    /// Build a lookup for work package hashes to segment roots
    async fn lookup(
        &self,
        work_package_hashes: &[WorkPackageHash],
    ) -> Result<BTreeMap<WorkPackageHash, OpaqueHash>> {
        let mut lookup = BTreeMap::new();
        for &work_package_hash in work_package_hashes {
            if let Some(segment_root) = self.segment_root(&work_package_hash).await? {
                lookup.insert(work_package_hash, segment_root);
            }
        }
        Ok(lookup)
    }

    /// Try to retrieve segment using page-proof justification
    async fn psegment(
        &self,
        segment_hash: &OpaqueHash,
        segments_root: &OpaqueHash,
        segment_index: u16,
    ) -> Result<Option<Segment>> {
        let page_index = segment_index / 64;
        let page_proof = self.page_proof(segments_root, page_index).await?;

        if let Some(proof) = page_proof {
            let segment_index_in_page = segment_index % 64;
            if let Some(segment) = self.segment(segment_hash).await? {
                if proof.verify_segment(&segment, segment_index_in_page)? {
                    return Ok(Some(segment));
                }
            }
        }
        Ok(None)
    }
}

/// In-memory segment provider for testing
#[derive(Default)]
pub struct InMemoryDataLake {
    segments: RwLock<HashMap<OpaqueHash, Segment>>,
    shards: RwLock<HashMap<OpaqueHash, Vec<Vec<u8>>>>,
    lookup: RwLock<HashMap<WorkPackageHash, OpaqueHash>>,
    page_proofs: RwLock<HashMap<(OpaqueHash, u16), PageProof>>,
}

impl DataLake for InMemoryDataLake {
    async fn segment(&self, segment_hash: &OpaqueHash) -> Result<Option<Segment>> {
        Ok(self.segments.read().await.get(segment_hash).copied())
    }

    async fn shards(&self, segment_hash: &OpaqueHash) -> Result<Option<Vec<Vec<u8>>>> {
        Ok(self.shards.read().await.get(segment_hash).cloned())
    }

    async fn set_segment(&self, segment_hash: OpaqueHash, segment: Segment) -> Result<()> {
        self.segments.write().await.insert(segment_hash, segment);
        Ok(())
    }

    async fn set_shards(&self, segment_hash: OpaqueHash, shards: Vec<Vec<u8>>) -> Result<()> {
        self.shards.write().await.insert(segment_hash, shards);
        Ok(())
    }

    async fn segment_root(&self, package: &WorkPackageHash) -> Result<Option<OpaqueHash>> {
        Ok(self.lookup.read().await.get(package).copied())
    }

    async fn register_work_package(
        &self,
        package: WorkPackageHash,
        segment_root: OpaqueHash,
    ) -> Result<()> {
        self.lookup.write().await.insert(package, segment_root);
        Ok(())
    }

    async fn set_page_proof(
        &self,
        segments_root: &OpaqueHash,
        page_index: u16,
        page_proof: PageProof,
    ) -> Result<()> {
        self.page_proofs
            .write()
            .await
            .insert((*segments_root, page_index), page_proof);
        Ok(())
    }

    async fn page_proof(
        &self,
        segments_root: &OpaqueHash,
        page_index: u16,
    ) -> Result<Option<PageProof>> {
        Ok(self
            .page_proofs
            .read()
            .await
            .get(&(*segments_root, page_index))
            .cloned())
    }
}
