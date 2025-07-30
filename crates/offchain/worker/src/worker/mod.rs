//! A worker that dispatched for a work package

use crate::SegmentProvider;
use score::service::WorkReport;

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
}

impl<P: SegmentProvider> Worker<P> {
    /// Create a new worker with a segment provider
    pub fn new(provider: P) -> Self {
        Self {
            report: WorkReport::default(),
            provider,
        }
    }
}
