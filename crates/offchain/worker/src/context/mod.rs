//! Spacejam work package builder
//! Work package computation

use score::{service::WorkReport, OpaqueHash};
use std::collections::BTreeMap;
use tokio::sync::RwLock;

mod package;
mod segment;

/// Context for the builder
#[derive(Default)]
pub struct Context {
    /// The reports of the builder
    pub reports: RwLock<BTreeMap<OpaqueHash, WorkReport>>,
}
