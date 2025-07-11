//! A worker that dispatched for a work package

use crate::Context;
use runtime::Config;
use score::service::WorkReport;
use std::sync::Arc;

mod authorize;
mod compute;
mod refine;
mod segment;

/// Worker for work package computation
pub struct Worker<C: Config> {
    /// Reference to the runtime (for future runtime functionality access)
    #[allow(dead_code)]
    context: Arc<Context<C>>,

    /// the computed work report
    report: WorkReport,
}

impl<C: Config> Worker<C> {
    /// Create a new worker
    pub fn new(context: Arc<Context<C>>) -> Self {
        Self {
            context,
            report: WorkReport::default(),
        }
    }
}
