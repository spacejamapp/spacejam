//! Segment provider trait and implementations

use anyhow::Result;
use score::{OpaqueHash, Segment, WorkPackageHash};
use std::collections::{BTreeMap, HashMap};
use tokio::sync::RwLock;

/// Trait for segment operations abstraction
/// Allows different implementations (in-memory, network-based, etc.)
#[allow(async_fn_in_trait)]
pub trait SegmentProvider: Send + Sync {
    /// Import segments by their hashes using erasure coding reconstruction
    async fn import_segments(&self, segment_hashes: &[OpaqueHash]) -> Result<Vec<Segment>>;

    /// Export segments to the data availability layer
    async fn export_segments(
        &self,
        segments: &[Segment],
        work_package_hash: &OpaqueHash,
    ) -> Result<OpaqueHash>;

    /// Check if segments are available by their hashes
    async fn segments_available(&self, segment_hashes: &[OpaqueHash]) -> Result<Vec<bool>>;

    /// Get segment root for a work-package hash
    /// Returns None if the work-package hash is not known
    async fn get_segment_root(
        &self,
        work_package_hash: &WorkPackageHash,
    ) -> Result<Option<OpaqueHash>>;

    /// Register a mapping from work-package hash to segment root
    async fn register_work_package(
        &self,
        work_package_hash: WorkPackageHash,
        segment_root: OpaqueHash,
    ) -> Result<()>;

    /// Build segment root lookup for a set of work-package hashes
    /// This is used to build the lookup dictionary for work reports
    async fn build_lookup(
        &self,
        work_package_hashes: &[WorkPackageHash],
    ) -> Result<BTreeMap<WorkPackageHash, OpaqueHash>>;
}

/// In-memory segment provider for testing
#[derive(Default)]
pub struct InMemorySegmentProvider {
    segments: RwLock<HashMap<OpaqueHash, Segment>>,
    shards: RwLock<HashMap<OpaqueHash, Vec<Vec<u8>>>>,
    bundles: RwLock<HashMap<OpaqueHash, Vec<Segment>>>,
    lookup: RwLock<HashMap<WorkPackageHash, OpaqueHash>>,
}

impl InMemorySegmentProvider {
    /// Store a segment with its erasure shards
    pub async fn store_segment(&self, segment_hash: OpaqueHash, segment: Segment) -> Result<()> {
        self.segments.write().await.insert(segment_hash, segment);
        let shards = erasure::encode_sync(segment.to_vec())?;
        self.shards.write().await.insert(segment_hash, shards);
        Ok(())
    }

    /// Store segments under a root hash (for bundle operations)
    pub async fn store_bundle(&self, root: OpaqueHash, segments: Vec<Segment>) {
        self.bundles.write().await.insert(root, segments);
    }

    /// Get segments by root hash
    pub async fn get_bundle(&self, root: &OpaqueHash) -> Option<Vec<Segment>> {
        self.bundles.read().await.get(root).cloned()
    }
}

impl SegmentProvider for InMemorySegmentProvider {
    async fn import_segments(&self, segment_hashes: &[OpaqueHash]) -> Result<Vec<Segment>> {
        let mut segments = Vec::new();

        for &hash in segment_hashes {
            // Try direct retrieval first
            if let Some(segment) = self.segments.read().await.get(&hash).copied() {
                segments.push(segment);
                continue;
            }

            // Try reconstruction from shards
            let shards = self.shards.read().await;
            if let Some(shard_data) = shards.get(&hash) {
                // Take minimum required shards for reconstruction
                let indexed_shards: Vec<(usize, Vec<u8>)> = shard_data
                    .iter()
                    .enumerate()
                    .take(shard_data.len() / 2)
                    .map(|(i, shard)| (i, shard.clone()))
                    .collect();

                let reconstructed = erasure::decode_sync(indexed_shards)?;
                if reconstructed.len() == score::SEGMENT_SIZE as usize {
                    let mut segment = [0u8; score::SEGMENT_SIZE as usize];
                    segment.copy_from_slice(&reconstructed);
                    segments.push(segment);
                } else {
                    return Err(anyhow::anyhow!(
                        "Reconstructed segment has wrong size: {} != {}",
                        reconstructed.len(),
                        score::SEGMENT_SIZE
                    ));
                }
            } else {
                return Err(anyhow::anyhow!("Segment not available: {:?}", hash));
            }
        }

        Ok(segments)
    }

    async fn export_segments(
        &self,
        segments: &[Segment],
        _work_package_hash: &OpaqueHash,
    ) -> Result<OpaqueHash> {
        if segments.is_empty() {
            return Ok([0u8; 32]);
        }

        let mut all_hashes = Vec::new();
        for segment in segments {
            let segment_hash = crypto::blake2b(segment);
            self.store_segment(segment_hash, *segment).await?;
            all_hashes.extend_from_slice(&segment_hash);
        }

        Ok(crypto::blake2b(&all_hashes))
    }

    async fn segments_available(&self, segment_hashes: &[OpaqueHash]) -> Result<Vec<bool>> {
        let segments = self.segments.read().await;
        let shards = self.shards.read().await;

        Ok(segment_hashes
            .iter()
            .map(|hash| segments.contains_key(hash) || shards.contains_key(hash))
            .collect())
    }

    async fn get_segment_root(
        &self,
        work_package_hash: &WorkPackageHash,
    ) -> Result<Option<OpaqueHash>> {
        Ok(self.lookup.read().await.get(work_package_hash).copied())
    }

    async fn register_work_package(
        &self,
        work_package_hash: WorkPackageHash,
        segment_root: OpaqueHash,
    ) -> Result<()> {
        self.lookup
            .write()
            .await
            .insert(work_package_hash, segment_root);
        Ok(())
    }

    async fn build_lookup(
        &self,
        work_package_hashes: &[WorkPackageHash],
    ) -> Result<BTreeMap<WorkPackageHash, OpaqueHash>> {
        let mappings = self.lookup.read().await;
        let mut lookup = BTreeMap::new();

        for &work_package_hash in work_package_hashes {
            if let Some(&segment_root) = mappings.get(&work_package_hash) {
                lookup.insert(work_package_hash, segment_root);
            }
        }

        Ok(lookup)
    }
}
