//! Work package bundle

use score::{OpaqueHash, service::WorkPackage};
use std::collections::HashMap;

/// Work package bundle
pub struct Bundle {
    /// The work package
    pub package: WorkPackage,

    /// The extrinsic data
    pub extrinsic: HashMap<OpaqueHash, Vec<u8>>,

    /// The segments
    pub segments: HashMap<OpaqueHash, Vec<OpaqueHash>>,
}
