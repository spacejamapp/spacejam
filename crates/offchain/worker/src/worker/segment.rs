//! segment related operations
//!
//! TODO: we may need to implement this in the network layer.
//! at that time, mb introduce a queue in runtime to do the
//! interaction.

use crate::Worker;
use anyhow::Result;
use runtime::Config;
use score::{service::WorkItem, OpaqueHash};

impl<C: Config> Worker<C> {
    /// Import segments for a work item using erasure coding reconstruction
    #[allow(dead_code)]
    pub fn import_segments<R: score::Accounts>(
        &self,
        item: &WorkItem,
        _accounts: &R,
    ) -> Result<Vec<[u8; score::SEGMENT_SIZE as usize]>> {
        // If no imports, return empty
        if item.import_segments.is_empty() {
            return Ok(vec![]);
        }

        // For now, return error if segments are actually needed
        Err(anyhow::anyhow!(
            "Segment imports not yet implemented - service requires segments"
        ))
    }

    /// Export segments to DA layer
    #[allow(dead_code)]
    pub fn export_segments(
        &self,
        exported_segments: &[[u8; score::SEGMENT_SIZE as usize]],
        _work_package_hash: &OpaqueHash,
    ) -> Result<OpaqueHash> {
        // If no exports, return empty root
        if exported_segments.is_empty() {
            return Ok([0u8; 32]);
        }

        // For now, return error if segments are actually exported
        Err(anyhow::anyhow!(
            "Segment exports not yet implemented - service tried to export segments"
        ))
    }
}
