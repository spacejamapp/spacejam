//! A worker that dispatched for a work package

use crate::{NetworkProvider, SegmentProvider};
use score::OpaqueHash;
use std::collections::HashMap;

mod authorize;
mod compute;
mod refine;
mod segment;

/// Worker for work package computation
pub struct Worker<S: SegmentProvider, N: NetworkProvider> {
    /// the segment provider
    segment_provider: S,

    /// the network provider
    network_provider: N,

    /// extrinsic data for the work package
    extrinsic_data: HashMap<OpaqueHash, Vec<u8>>,
}

impl<S: SegmentProvider, N: NetworkProvider> Worker<S, N> {
    /// Create a new worker with a segment provider and network provider
    pub fn new(segment_provider: S, network_provider: N) -> Self {
        Self {
            segment_provider,
            network_provider,
            extrinsic_data: HashMap::new(),
        }
    }

    /// Create a new worker with providers and extrinsic data
    pub fn with_extrinsics(
        segment_provider: S,
        network_provider: N,
        extrinsic_data: HashMap<OpaqueHash, Vec<u8>>,
    ) -> Self {
        Self {
            segment_provider,
            network_provider,
            extrinsic_data,
        }
    }

    /// Set extrinsic data for the worker
    pub fn set_extrinsics(&mut self, extrinsic_data: HashMap<OpaqueHash, Vec<u8>>) {
        self.extrinsic_data = extrinsic_data;
    }
}
