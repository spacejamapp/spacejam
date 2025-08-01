//! Segment provider trait and implementations

use crate::segment::{shard, BundleShardJustification, SegmentShardJustification};
use anyhow::Result;
use score::{OpaqueHash, Segment, WorkPackageHash};
use std::collections::{BTreeMap, HashMap};
use tokio::sync::RwLock;

/// Trait for segment operations abstraction
/// Allows different implementations (in-memory, network-based, etc.)
#[allow(async_fn_in_trait)]
pub trait SegmentProvider: Send + Sync {
    /// Get a segment
    async fn segment(&self, segment_hash: &OpaqueHash) -> Result<Option<Segment>>;

    /// Get shards
    async fn shards(&self, segment_hash: &OpaqueHash) -> Result<Option<Vec<Vec<u8>>>>;

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

    /// Export segments
    async fn export_segments(
        &self,
        segments: &[Segment],
        _work_package_hash: &OpaqueHash,
    ) -> Result<OpaqueHash> {
        if segments.is_empty() {
            return Ok([0u8; 32]);
        }

        let mut all_hashes = Vec::with_capacity(segments.len() * 32);
        for segment in segments {
            let segment_hash = crypto::blake2b(segment);
            let shards = erasure::encode(segment.to_vec()).await?;

            // Store both segment and shards concurrently if possible
            self.set_segment(segment_hash, *segment).await?;
            self.set_shards(segment_hash, shards).await?;

            all_hashes.extend_from_slice(&segment_hash);
        }

        Ok(crypto::blake2b(&all_hashes))
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
    async fn import_segments_with_validation(
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

    /// Get the justification for a segment shard - with smart orchestration
    async fn segment_justification(
        &self,
        erasure_root: &OpaqueHash,
        segment_index: u16,
        shard_index: u16,
    ) -> Result<Option<SegmentShardJustification>> {
        // Try cache first, then compute and cache
        if let Some(cached) = self
            .get_segment_justification(erasure_root, segment_index, shard_index)
            .await?
        {
            return Ok(Some(cached));
        }

        let Some(shards) = self.shards(erasure_root).await? else {
            return Ok(None);
        };

        let justification =
            SegmentShardJustification::new(&shards, erasure_root, segment_index, shard_index)?;
        if let Some(ref j) = justification {
            self.set_segment_justification(*erasure_root, segment_index, shard_index, j.clone())
                .await?;
        }
        Ok(justification)
    }

    /// Get the justification for a bundle shard - with smart orchestration
    async fn bundle_justification(
        &self,
        erasure_root: &OpaqueHash,
        shard_index: u16,
    ) -> Result<Option<BundleShardJustification>> {
        // Try cache first, then compute and cache
        if let Some(cached) = self
            .get_bundle_justification(erasure_root, shard_index)
            .await?
        {
            return Ok(Some(cached));
        }

        let Some(shards) = self.shards(erasure_root).await? else {
            return Ok(None);
        };

        let justification = BundleShardJustification::new(&shards, erasure_root, shard_index)?;
        if let Some(ref j) = justification {
            self.set_bundle_justification(*erasure_root, shard_index, j.clone())
                .await?;
        }
        Ok(justification)
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

    /// Cache lookup hook for segment justifications
    async fn get_segment_justification(
        &self,
        _erasure_root: &OpaqueHash,
        _segment_index: u16,
        _shard_index: u16,
    ) -> Result<Option<SegmentShardJustification>> {
        Ok(None)
    }

    /// Cache lookup hook for bundle justifications
    async fn get_bundle_justification(
        &self,
        _erasure_root: &OpaqueHash,
        _shard_index: u16,
    ) -> Result<Option<BundleShardJustification>> {
        Ok(None)
    }

    /// set a segment justification
    async fn set_segment_justification(
        &self,
        _erasure_root: OpaqueHash,
        _segment_index: u16,
        _shard_index: u16,
        _justification: SegmentShardJustification,
    ) -> Result<()> {
        Ok(())
    }

    /// set a bundle justification
    async fn set_bundle_justification(
        &self,
        _erasure_root: OpaqueHash,
        _shard_index: u16,
        _justification: BundleShardJustification,
    ) -> Result<()> {
        Ok(())
    }
}

/// In-memory segment provider for testing
#[derive(Default)]
pub struct InMemorySegmentProvider {
    segments: RwLock<HashMap<OpaqueHash, Segment>>,
    shards: RwLock<HashMap<OpaqueHash, Vec<Vec<u8>>>>,
    lookup: RwLock<HashMap<WorkPackageHash, OpaqueHash>>,
}

impl SegmentProvider for InMemorySegmentProvider {
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
}
