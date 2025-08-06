//! Assurer abstraction

use crate::{d3l::Justification, DataLake};
use anyhow::Result;
use score::{service::WorkReport, OpaqueHash, Segment};

/// Assurer abstraction
#[allow(async_fn_in_trait)]
pub trait Assurer: DataLake {
    /// Get a work report by hash
    async fn work_report(&self, report_hash: OpaqueHash) -> Result<WorkReport> {
        self.get_work_report(&report_hash)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Work report not found: {:?}", report_hash))
    }

    /// Get audit shard data with justification
    async fn audit_shard(
        &self,
        erasure_root: OpaqueHash,
        shard_index: u16,
    ) -> Result<(Vec<u8>, Justification)> {
        // Get the bundle shard
        let bundle_shard = self
            .get_shard(&erasure_root, shard_index)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Bundle shard not found: {:?} index {}",
                    erasure_root,
                    shard_index
                )
            })?;

        // Get justification for the bundle shard
        let justification = self
            .bundle_justification(&erasure_root, shard_index)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Could not generate bundle justification for shard {} of {:?}",
                    shard_index,
                    erasure_root
                )
            })?;

        // Extract the justification path (first justification in the path)
        let justification = justification
            .path
            .path
            .get(0)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Empty justification path for shard {} of {:?}",
                    shard_index,
                    erasure_root
                )
            })?
            .clone();


        Ok((bundle_shard, justification))
    }

    /// Get segment shard data with justifications
    async fn segment(
        &self,
        erasure_root: OpaqueHash,
        shard_index: u16,
        segment_indices: Vec<u16>,
    ) -> Result<Vec<(Segment, Justification)>> {
        // Default to including justifications
        let results = self
            .segment_shards(erasure_root, shard_index, segment_indices, true)
            .await?;

        // Convert to expected return type
        results
            .into_iter()
            .map(|(shard_data, justification)| {
                let segment: Segment = if shard_data.len() == score::SEGMENT_SIZE as usize {
                    let mut segment = [0u8; score::SEGMENT_SIZE as usize];
                    segment.copy_from_slice(&shard_data);
                    segment
                } else {
                    // If not proper segment size, zero-pad or truncate as needed
                    let mut segment = [0u8; score::SEGMENT_SIZE as usize];
                    let copy_len = shard_data.len().min(score::SEGMENT_SIZE as usize);
                    segment[..copy_len].copy_from_slice(&shard_data[..copy_len]);
                    segment
                };

                let justification = justification.unwrap_or(Justification::Hash([0u8; 32]));
                Ok((segment, justification))
            })
            .collect()
    }

    /// Get segment shard data with optional justification control
    async fn segment_shards(
        &self,
        erasure_root: OpaqueHash,
        shard_index: u16,
        segment_indices: Vec<u16>,
        with_justification: bool,
    ) -> Result<Vec<(Vec<u8>, Option<Justification>)>> {

        // Get all shards for the erasure root
        let all_shards = self.get_shards(&erasure_root).await?.ok_or_else(|| {
            anyhow::anyhow!("No shards found for erasure root: {:?}", erasure_root)
        })?;

        // Check that shard_index is valid
        if (shard_index as usize) >= all_shards.len() {
            return Err(anyhow::anyhow!(
                "Shard index {} out of bounds for {} shards",
                shard_index,
                all_shards.len()
            ));
        }

        let mut results = Vec::with_capacity(segment_indices.len());
        for segment_index in segment_indices {
            // For now, return the shard data at the shard_index
            // TODO: Properly separate bundle vs segment shards based on erasure layout
            let segment_shard = all_shards
                .get(shard_index as usize)
                .ok_or_else(|| anyhow::anyhow!("Shard {} not available", shard_index))?
                .clone();

            let justification = if with_justification {
                // Generate justification if requested
                self.segment_justification(&erasure_root, segment_index, shard_index)
                    .await?
                    .and_then(|sj| sj.path.path.get(0).cloned())
            } else {
                // No justification
                None
            };

            results.push((segment_shard, justification));
        }

        Ok(results)
    }
}
