//! Work package computation

use crate::{Config, Runtime};
use anyhow::Result;
use score::service::{WorkPackage, WorkReport};

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

impl<C: Config> Runtime<C> {
    /// Compute the work package using a worker
    pub fn compute<R: score::Accounts>(
        &self,
        work: WorkPackage,
        core_idx: usize,
        accounts: R,
    ) -> Result<WorkReport> {
        Worker::new(self).compute(work, core_idx, accounts)
    }
}
