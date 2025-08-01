//! Worker segment operations

use crate::{NetworkProvider, SegmentProvider, Worker};
use anyhow::Result;
use score::{service::WorkItem, OpaqueHash, Segment};

impl<S: SegmentProvider, N: NetworkProvider> Worker<S, N> {
    /// Import segments for a work item
    pub async fn import_segments(&self, item: &WorkItem) -> Result<Vec<Segment>> {
        if item.import_segments.is_empty() {
            return Ok(vec![]);
        }

        let segment_hashes: Vec<_> = item
            .import_segments
            .iter()
            .map(|spec| spec.tree_root)
            .collect();

        self.segment_provider.import_segments(&segment_hashes).await
    }

    /// Export segments for a work package
    pub async fn export_segments(
        &self,
        segments: &[Segment],
        work_package_hash: &OpaqueHash,
    ) -> Result<OpaqueHash> {
        if segments.is_empty() {
            return Ok([0u8; 32]);
        }

        self.segment_provider
            .export_segments(segments, work_package_hash)
            .await
    }
}
