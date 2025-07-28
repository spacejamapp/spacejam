//! Spacejam work package builder
//! Work package computation

use score::{service::WorkReport, OpaqueHash};
use std::collections::BTreeMap;
use tokio::sync::RwLock;

mod package;
mod segment;

/// Context for the builder
pub struct Context {
    /// The reports of the builder
    pub reports: RwLock<BTreeMap<OpaqueHash, WorkReport>>,
}

impl Context {
    /// Create a new context
    pub fn new() -> Self {
        Self {
            reports: RwLock::new(BTreeMap::new()),
        }
    }
}
