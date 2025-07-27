//! Spacejam work package builder
//! Work package computation

use network::Network;
use runtime::Config;
use score::{service::WorkReport, OpaqueHash};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::RwLock;

mod package;
mod segment;

/// Context for the builder
pub struct Context<C: Config> {
    /// Reference to the network (for future network functionality access)
    pub network: Arc<Network<C>>,

    /// The reports of the builder
    pub reports: RwLock<BTreeMap<OpaqueHash, WorkReport>>,
}

impl<C: Config> Context<C> {
    /// Create a new context
    pub fn new(network: Arc<Network<C>>) -> Self {
        Self {
            network,
            reports: RwLock::new(BTreeMap::new()),
        }
    }
}
