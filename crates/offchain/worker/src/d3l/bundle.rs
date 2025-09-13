//! Work package bundle

use crate::d3l::Justification;
use score::{OpaqueHash, service::WorkPackage};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Work package bundle
#[derive(Serialize, Deserialize)]
pub struct WorkPackageBundle {
    /// The work package itself
    pub package: WorkPackage,

    /// The extrinsic data
    ///
    /// TODO: Vec instead of Map?
    pub extrinsic: BTreeMap<OpaqueHash, Vec<u8>>,

    /// The concatenated import segments along with their proofs of correctness
    pub imports_with_proofs: Vec<(Vec<u8>, Vec<Justification>)>,
}

impl WorkPackageBundle {
    /// Create a new work package bundle with default empty collections
    pub fn new(package: score::service::WorkPackage) -> Self {
        Self {
            package,
            extrinsic: Default::default(),
            imports_with_proofs: Default::default(),
        }
    }
}
