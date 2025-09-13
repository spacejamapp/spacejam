//! Assurer abstraction

use crate::{DataLake, d3l::Justification};
use anyhow::Result;
use score::{OpaqueHash, Segment, service::WorkReport};

/// Assurer abstraction
#[allow(async_fn_in_trait)]
pub trait Assurer: DataLake {
    /// Get a work report by hash
    async fn work_report(&self, report_hash: OpaqueHash) -> Result<WorkReport> {
        self.get_work_report(&report_hash)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Work report not found: {:?}", report_hash))
    }

    /// Get audit shard data with justification (CE138)
    async fn audit_shard(
        &self,
        erasure_root: OpaqueHash,
        shard_index: u16,
    ) -> Result<(Vec<u8>, Justification)> {
        // Get the bundle shard (not segment shard)
        let bundle_shard = self
            .get_bundle_shard(&erasure_root, shard_index)
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
            .first()
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

    /// Get segment shard data with optional justification control (CE139/CE140)
    async fn segment_shards(
        &self,
        erasure_root: OpaqueHash,
        shard_index: u16,
        segment_indices: Vec<u16>,
        with_justification: bool,
    ) -> Result<Vec<(Segment, Option<Justification>)>> {
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

        // Get the shard layout to properly separate bundle vs segment shards
        let mut results = Vec::with_capacity(segment_indices.len());
        let layout = self.get_shard_layout(&erasure_root).await?.ok_or_else(|| {
            anyhow::anyhow!(
                "Shard layout not found for erasure root: {:?}",
                erasure_root
            )
        })?;

        // TODO: verify the count of the chunks
        let (bundle_count, _) = layout;
        let segment_shards_start = bundle_count;
        for segment_index in segment_indices {
            let segment_shard_idx = segment_shards_start + (shard_index as usize);
            let segment_shard = all_shards
                .get(segment_shard_idx)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Segment shard {} not available at index {}",
                        shard_index,
                        segment_shard_idx
                    )
                })?
                .clone();

            // Embed justification if requested
            let justification = if with_justification {
                self.segment_justification(&erasure_root, segment_index, shard_index)
                    .await?
                    .and_then(|sj| sj.path.path.first().cloned())
            } else {
                None
            };

            let mut segment = [0u8; score::SEGMENT_SIZE as usize];
            segment.copy_from_slice(&segment_shard);
            results.push((segment, justification));
        }

        Ok(results)
    }
}

impl<T: DataLake> Assurer for T {}
