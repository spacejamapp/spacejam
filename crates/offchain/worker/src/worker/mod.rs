//! A worker that dispatched for a work package

use crate::DataLake;
use score::OpaqueHash;
use std::collections::HashMap;

mod authorize;
mod compute;
mod refine;
mod segment;

/// Worker for work package computation - simplified without network dependencies
///
/// In the refined architecture, Worker is generic over SegmentProvider only.
/// Network operations are handled by the Network library calling Runtime methods directly.
pub struct Worker<S: DataLake> {
    /// the segment provider
    segment_provider: S,

    /// extrinsic data for the work package
    extrinsic_data: HashMap<OpaqueHash, Vec<u8>>,
}

impl<S: DataLake> Worker<S> {
    /// Create a new worker with a segment provider
    pub fn new(segment_provider: S) -> Self {
        Self {
            segment_provider,
            extrinsic_data: HashMap::new(),
        }
    }

    /// Create a new worker with segment provider and extrinsic data
    pub fn with_extrinsics(
        segment_provider: S,
        extrinsic_data: HashMap<OpaqueHash, Vec<u8>>,
    ) -> Self {
        Self {
            segment_provider,
            extrinsic_data,
        }
    }

    /// Set extrinsic data for the worker
    pub fn set_extrinsics(&mut self, extrinsic_data: HashMap<OpaqueHash, Vec<u8>>) {
        self.extrinsic_data = extrinsic_data;
    }
}
