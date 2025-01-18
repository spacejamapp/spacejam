//! Temporary dependencies for validation

use crate::guarantee::{Error, Result};
use score::{extrinsic::ReportGuarantee, work::ReportedWorkPackage, OpaqueHash};

/// Temp dependencies for validation
#[derive(Default)]
pub struct Dependencies {
    pub service: Vec<OpaqueHash>,
    pub recent: Vec<ReportedWorkPackage>,
    pub reported: Vec<OpaqueHash>,
}

impl Dependencies {
    /// Check if the dependencies contains a hash
    pub fn contains(&self, hash: &OpaqueHash) -> bool {
        self.service.contains(hash)
            || self.recent.iter().any(|r| r.hash == *hash)
            || self.reported.contains(hash)
    }

    // TODO: check if duplicated in service deps?
    pub fn duplicated(&self, hash: &OpaqueHash) -> bool {
        self.recent.iter().any(|r| r.hash == *hash)
            || self.reported.iter().filter(|h| *h == hash).count() > 1
    }

    /// Validate segment lookup
    pub fn validate_segment_lookup(&self, guarantee: &ReportGuarantee) -> Result<()> {
        for lookup in guarantee.report.segment_root_lookup.iter() {
            if self.reported.contains(&lookup.work_package_hash) {
                continue;
            }

            let Some(reported) = self
                .recent
                .iter()
                .find(|r| r.hash == lookup.work_package_hash)
            else {
                return Err(Error::SegmentRootLookupInvalid);
            };

            if reported.exports_root != lookup.segment_tree_root {
                return Err(Error::SegmentRootLookupInvalid);
            }
        }
        Ok(())
    }
}
