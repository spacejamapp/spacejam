//! A worker that dispatched for a work package

use crate::SegmentProvider;
use score::{service::WorkReport, OpaqueHash};
use std::collections::HashMap;

mod authorize;
mod compute;
mod refine;
mod segment;

/// Worker for work package computation
pub struct Worker<P: SegmentProvider> {
    /// the computed work report
    pub report: WorkReport,

    /// the segment provider
    provider: P,

    /// extrinsic data for the work package
    extrinsic_data: HashMap<OpaqueHash, Vec<u8>>,
}

impl<P: SegmentProvider> Worker<P> {
    /// Create a new worker with a segment provider
    pub fn new(provider: P) -> Self {
        Self {
            report: WorkReport::default(),
            provider,
            extrinsic_data: HashMap::new(),
        }
    }

    /// Create a new worker with a segment provider and extrinsic data
    pub fn with_extrinsics(provider: P, extrinsic_data: HashMap<OpaqueHash, Vec<u8>>) -> Self {
        Self {
            report: WorkReport::default(),
            provider,
            extrinsic_data,
        }
    }

    /// Set extrinsic data for the worker
    pub fn set_extrinsics(&mut self, extrinsic_data: HashMap<OpaqueHash, Vec<u8>>) {
        self.extrinsic_data = extrinsic_data;
    }
}
