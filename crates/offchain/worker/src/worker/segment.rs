//! Worker segment operations

use crate::{SegmentProvider, Worker};
use anyhow::Result;
use score::{service::WorkItem, OpaqueHash, Segment};

impl<P: SegmentProvider> Worker<P> {
    /// Import segments for a work item using provider
    pub async fn import_segments_with_provider(&self, item: &WorkItem) -> Result<Vec<Segment>> {
        if item.import_segments.is_empty() {
            return Ok(vec![]);
        }

        // Extract tree roots from import specs
        let segment_hashes: Vec<OpaqueHash> = item
            .import_segments
            .iter()
            .map(|spec| spec.tree_root)
            .collect();

        self.provider.import_segments(&segment_hashes).await
    }

    /// Export segments using provider
    pub async fn export_segments_with_provider(
        &self,
        exported_segments: &[Segment],
        work_package_hash: &OpaqueHash,
        provider: &P,
    ) -> Result<OpaqueHash> {
        if exported_segments.is_empty() {
            return Ok([0u8; 32]);
        }

        provider
            .export_segments(exported_segments, work_package_hash)
            .await
    }

    /// Legacy import method
    #[allow(dead_code)]
    pub fn import_segments<R: score::Accounts>(
        &self,
        item: &WorkItem,
        _accounts: &R,
    ) -> Result<Vec<Segment>> {
        if item.import_segments.is_empty() {
            return Ok(vec![]);
        }

        Err(anyhow::anyhow!(
            "Segment imports not yet implemented - use import_segments_with_provider instead"
        ))
    }

    /// Legacy export method
    #[allow(dead_code)]
    pub fn export_segments(
        &self,
        exported_segments: &[Segment],
        _work_package_hash: &OpaqueHash,
    ) -> Result<OpaqueHash> {
        if exported_segments.is_empty() {
            return Ok([0u8; 32]);
        }

        Err(anyhow::anyhow!(
            "Segment exports not yet implemented - use export_segments_with_provider instead"
        ))
    }
}
