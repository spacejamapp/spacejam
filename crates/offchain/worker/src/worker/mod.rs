//! A worker that dispatched for a work package

use score::service::WorkReport;

mod authorize;
mod compute;
mod refine;
mod segment;

/// Worker for work package computation
#[derive(Default)]
pub struct Worker {
    /// the computed work report
    pub report: WorkReport,
}
