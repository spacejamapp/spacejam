//! Data lake provider trait and implementations

use crate::d3l::{shard, BundleShardJustification, PageProof, SegmentShardJustification};
use crate::WorkPackageBundle;
use anyhow::Result;
use score::{service::WorkPackageSpec, OpaqueHash, Segment, WorkPackageHash};
use std::collections::{BTreeMap, HashMap};
use tokio::sync::RwLock;

/// Trait for data lake operations abstraction
#[allow(async_fn_in_trait)]
pub trait DataLake: Send + Sync {
    /// Get a segment by hash
    async fn get_segment(&self, segment_hash: &OpaqueHash) -> Result<Option<Segment>>;

    /// Store a segment
    async fn store_segment(&self, segment_hash: OpaqueHash, segment: Segment) -> Result<()>;

    /// Get all shards for an erasure root
    async fn get_shards(&self, erasure_root: &OpaqueHash) -> Result<Option<Vec<Vec<u8>>>>;

    /// Store shards for an erasure root
    async fn store_shards(&self, erasure_root: OpaqueHash, shards: Vec<Vec<u8>>) -> Result<()>;

    /// Get a specific shard by erasure root and shard index
    async fn get_shard(
        &self,
        erasure_root: &OpaqueHash,
        shard_index: u16,
    ) -> Result<Option<Vec<u8>>> {
        let shards = self.get_shards(erasure_root).await?;
        Ok(shards.and_then(|s| s.get(shard_index as usize).cloned()))
    }

    /// Get the segment root for a work package hash
    async fn get_segment_root(
        &self,
        work_package_hash: &WorkPackageHash,
    ) -> Result<Option<OpaqueHash>>;

    /// Register a work package with its segment root
    async fn register_work_package(
        &self,
        work_package_hash: WorkPackageHash,
        segment_root: OpaqueHash,
    ) -> Result<()>;

    /// Get page-proof for segment justification
    async fn get_page_proof(
        &self,
        segments_root: &OpaqueHash,
        page_index: u16,
    ) -> Result<Option<PageProof>>;

    /// Store page-proof for efficient segment justification
    async fn store_page_proof(
        &self,
        segments_root: &OpaqueHash,
        page_index: u16,
        page_proof: PageProof,
    ) -> Result<()>;

    /// Compute availability specification and store associated shards
    async fn specify_bundle(
        &self,
        bundle: &WorkPackageBundle,
        exported: Vec<Segment>,
    ) -> Result<WorkPackageSpec> {
        // 1. Generate segment chunks (s♣) and page proofs
        let exports_root = crypto::blake2b(
            exported
                .iter()
                .flatten()
                .copied()
                .collect::<Vec<_>>()
                .as_slice(),
        );
        let proofs = PageProof::proofs(&exported, &exports_root).await?;
        let encoded = codec::encode(&(
            exported.to_vec().iter().flatten().collect::<Vec<_>>(),
            proofs,
        ))?;
        let exported_chunks = erasure::encode(encoded).await?;
        let exported_chunk_hashes: Vec<OpaqueHash> = exported_chunks
            .iter()
            .map(|chunk| crypto::blake2b(chunk))
            .collect();

        // 2. Generate bundle chunks (s♣)
        let bundle_data = codec::encode(bundle)?;
        let length = bundle_data.len() as u32;
        let bundle_chunks = erasure::encode(bundle_data).await?;
        let bundle_chunk_hashes: Vec<OpaqueHash> = bundle_chunks
            .iter()
            .map(|chunk| crypto::blake2b(chunk))
            .collect();

        // 3. Get merkle root of all chunks
        let chunks = bundle_chunk_hashes.len().max(exported_chunk_hashes.len());
        let leaves: Vec<Vec<u8>> = (0..chunks)
            .map(|i| {
                let mut leaf = Vec::new();
                if let Some(bundle_chunk) = bundle_chunk_hashes.get(i) {
                    leaf.extend_from_slice(bundle_chunk);
                }
                if let Some(exported_chunk) = exported_chunk_hashes.get(i) {
                    leaf.extend_from_slice(exported_chunk);
                }
                leaf
            })
            .collect();

        let erasure_root = crypto::merkle::root(&leaves);

        // 4. Store shards for later retrieval
        let mut all_shards = Vec::new();
        all_shards.extend(bundle_chunks);
        all_shards.extend(exported_chunks);
        self.store_shards(erasure_root, all_shards).await?;

        // 5. Store exported segments for import
        for segment in &exported {
            let segment_hash = crypto::blake2b(segment);
            self.store_segment(segment_hash, *segment).await?;
        }

        Ok(WorkPackageSpec {
            hash: crypto::blake2b(&codec::encode(&bundle.package)?),
            length,
            erasure_root,
            exports_root: crypto::blake2b(
                exported_chunk_hashes
                    .iter()
                    .flatten()
                    .copied()
                    .collect::<Vec<_>>()
                    .as_slice(),
            ),
            exports_count: exported.len() as u16,
        })
    }

    /// Check if segments are available (with default implementation)
    async fn segments_available(&self, segment_hashes: &[OpaqueHash]) -> Result<Vec<bool>> {
        let mut availability = Vec::with_capacity(segment_hashes.len());
        for &hash in segment_hashes {
            let available = self.get_segment(&hash).await?.is_some()
                || self
                    .get_shards(&hash)
                    .await?
                    .is_some_and(|shards| shards.len() >= shard::min_shards());
            availability.push(available);
        }
        Ok(availability)
    }

    /// Import segments (with default implementation)
    async fn import_segments(&self, segment_hashes: &[OpaqueHash]) -> Result<Vec<Segment>> {
        let mut segments = Vec::with_capacity(segment_hashes.len());
        for &hash in segment_hashes {
            let segment = match (
                self.get_segment(&hash).await?,
                self.get_shards(&hash).await?,
            ) {
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

    /// Build a lookup for work package hashes to segment roots
    async fn lookup(
        &self,
        work_package_hashes: &[WorkPackageHash],
    ) -> Result<BTreeMap<WorkPackageHash, OpaqueHash>> {
        let mut lookup = BTreeMap::new();
        for &work_package_hash in work_package_hashes {
            if let Some(segment_root) = self.get_segment_root(&work_package_hash).await? {
                lookup.insert(work_package_hash, segment_root);
            }
        }
        Ok(lookup)
    }

    /// Get justification for a bundle shard
    async fn bundle_justification(
        &self,
        erasure_root: &OpaqueHash,
        shard_index: u16,
    ) -> Result<Option<BundleShardJustification>> {
        let Some(shards) = self.get_shards(erasure_root).await? else {
            return Ok(None);
        };
        BundleShardJustification::new(&shards, erasure_root, shard_index)
    }

    /// Get justification for a segment shard
    async fn segment_justification(
        &self,
        erasure_root: &OpaqueHash,
        segment_index: u16,
        shard_index: u16,
    ) -> Result<Option<SegmentShardJustification>> {
        let Some(shards) = self.get_shards(erasure_root).await? else {
            return Ok(None);
        };
        SegmentShardJustification::new(&shards, erasure_root, segment_index, shard_index)
    }
}

/// In-memory data lake for testing
#[derive(Default)]
pub struct InMemoryDataLake {
    segments: RwLock<HashMap<OpaqueHash, Segment>>,
    shards: RwLock<HashMap<OpaqueHash, Vec<Vec<u8>>>>,
    lookup: RwLock<HashMap<WorkPackageHash, OpaqueHash>>,
    page_proofs: RwLock<HashMap<(OpaqueHash, u16), PageProof>>,
}

impl DataLake for InMemoryDataLake {
    async fn get_segment(&self, segment_hash: &OpaqueHash) -> Result<Option<Segment>> {
        Ok(self.segments.read().await.get(segment_hash).copied())
    }

    async fn store_segment(&self, segment_hash: OpaqueHash, segment: Segment) -> Result<()> {
        self.segments.write().await.insert(segment_hash, segment);
        Ok(())
    }

    async fn get_shards(&self, erasure_root: &OpaqueHash) -> Result<Option<Vec<Vec<u8>>>> {
        Ok(self.shards.read().await.get(erasure_root).cloned())
    }

    async fn store_shards(&self, erasure_root: OpaqueHash, shards: Vec<Vec<u8>>) -> Result<()> {
        self.shards.write().await.insert(erasure_root, shards);
        Ok(())
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

    async fn get_page_proof(
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

    async fn store_page_proof(
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
}
