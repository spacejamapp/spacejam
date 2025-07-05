//! Spacejam work package builder
//! Work package computation

use runtime::{Config, Runtime};
use score::service::WorkReport;

mod authorize;
mod compute;
mod refine;
mod segment;

/// Worker for work package computation
pub struct Worker<'a, C: Config> {
    /// Reference to the runtime (for future runtime functionality access)
    #[allow(dead_code)]
    runtime: &'a Runtime<C>,

    /// the computed work report
    report: WorkReport,
}

impl<'a, C: Config> Worker<'a, C> {
    /// Create a new worker
    pub fn new(runtime: &'a Runtime<C>) -> Self {
        Self {
            runtime,
            report: WorkReport::default(),
        }
    }
}
