//! Work package bundle

use score::{service::WorkPackage, OpaqueHash, WorkPackageHash};
use std::collections::HashMap;

/// Work package bundle
pub struct WorkPackageBundle {
    /// The work package
    pub package: WorkPackage,

    /// The extrinsic data
    pub extrinsic: HashMap<OpaqueHash, Vec<u8>>,

    /// The segments
    pub segments: HashMap<OpaqueHash, Vec<OpaqueHash>>,

    /// Mapping from work-package hash to segment root
    /// This is used when imports reference work-package hashes (h⊞) instead of segment roots
    pub segment_roots: HashMap<WorkPackageHash, OpaqueHash>,
}

impl WorkPackageBundle {
    /// Create a new work package bundle with default empty collections
    pub fn new(package: score::service::WorkPackage) -> Self {
        Self {
            package,
            extrinsic: Default::default(),
            segments: Default::default(),
            segment_roots: Default::default(),
        }
    }
}
