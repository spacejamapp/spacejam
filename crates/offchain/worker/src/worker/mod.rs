//! A worker that dispatched for a work package

use score::service::WorkReport;

mod authorize;
mod compute;
mod refine;
mod segment;

/// Worker for work package computation
pub struct Worker {
    /// the computed work report
    pub report: WorkReport,
}

impl Worker {
    /// Create a new worker
    pub fn new() -> Self {
        Self {
            report: WorkReport::default(),
        }
    }
}
