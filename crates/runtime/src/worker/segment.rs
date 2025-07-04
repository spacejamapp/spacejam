//! segment related operations

use crate::{
    worker::{SegmentRootLookupResult, Worker},
    Config,
};
use anyhow::Result;
use score::{
    service::{ReportedWorkPackage, WorkItem, WorkPackage},
    OpaqueHash,
};
use std::collections::{hash_map::Entry, HashMap};

impl<'a, C: Config> Worker<'a, C> {
    // Phase 2: Build segment root lookup directory
    pub fn build_segment_root_lookup<R: score::Accounts>(
        &self,
        work: &WorkPackage,
        accounts: &R,
    ) -> Result<SegmentRootLookupResult> {
        let mut segment_root_lookup = Vec::new();
        let mut segment_lookup_map = HashMap::new();

        // Process all work items to collect segment root lookups
        for item in &work.items {
            for import in &item.import_segments {
                // Check if this is a work package hash reference (needs lookup)
                if let Some(work_package_hash) = self.extract_work_package_hash(&import.tree_root) {
                    if let Entry::Vacant(e) = segment_lookup_map.entry(work_package_hash) {
                        let segment_root =
                            self.lookup_segment_root(&work_package_hash, accounts)?;
                        e.insert(segment_root);
                        segment_root_lookup.push(ReportedWorkPackage {
                            hash: work_package_hash,
                            exports_root: segment_root,
                        });
                    }
                }
            }
        }

        Ok(SegmentRootLookupResult {
            segment_root_lookup,
            segment_lookup_map,
        })
    }

    /// Lookup segment root for a work package hash
    fn lookup_segment_root<R: score::Accounts>(
        &self,
        _work_package_hash: &OpaqueHash,
        _accounts: &R,
    ) -> Result<OpaqueHash> {
        // TODO: Implement segment root lookup
        // This would involve looking up previous work reports and their segment roots
        anyhow::bail!("Segment root lookup not yet implemented")
    }

    /// Import segments for a work item using erasure coding reconstruction
    pub fn import_segments<R: score::Accounts>(
        &self,
        item: &WorkItem,
        segment_lookup_map: &HashMap<OpaqueHash, OpaqueHash>,
        _accounts: &R,
    ) -> Result<Vec<[u8; score::SEGMENT_SIZE as usize]>> {
        let mut imported_segments = Vec::new();

        for import_spec in &item.import_segments {
            // Resolve the actual segment root
            let segment_root = segment_lookup_map
                .get(&import_spec.tree_root)
                .unwrap_or(&import_spec.tree_root);

            // TODO: Implement actual segment import with erasure coding
            // This would involve:
            // 1. Fetching erasure-coded chunks from validators
            // 2. Reconstructing the segment using erasure::decode
            // 3. Verifying the segment against the Merkle proof

            tracing::debug!(
                "Importing segment {} from root {:?}",
                import_spec.index,
                segment_root
            );

            // Placeholder: return empty segment for now
            imported_segments.push([0u8; score::SEGMENT_SIZE as usize]);
        }

        Ok(imported_segments)
    }

    /// Extract work package hash from tree root if it's a tagged variant
    pub fn extract_work_package_hash(&self, _tree_root: &OpaqueHash) -> Option<OpaqueHash> {
        // TODO: Implement proper tagged variant detection
        // For now, assume all are direct segment roots
        None
    }
}
