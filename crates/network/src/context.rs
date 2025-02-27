//! Context for the network.

use crate::peer::Manager;
use crypto::ed25519;
use metrics::Metrics;
use score::runtime::Grandpa;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{broadcast, mpsc, RwLock};

/// Context for the network.
#[allow(async_fn_in_trait)]
pub trait Context {
    /// Get the keypair of the p2p network.
    fn keypair(&self) -> Option<ed25519::KeyPair> {
        None
    }

    /// Get the metrics of the node.
    fn metrics(&self) -> &Metrics;

    /// Announce the handshake message.
    fn grandpa(&self) -> Grandpa;

    /// Get the manager of the network.
    fn manager(&self) -> Arc<RwLock<Manager>>;
}

impl Context for Metrics {
    fn metrics(&self) -> &Metrics {
        self
    }

    fn grandpa(&self) -> Grandpa {
        Arc::new(RwLock::new(Default::default()))
    }

    fn manager(&self) -> Arc<RwLock<Manager>> {
        Arc::new(RwLock::new(Manager {
            conns: HashMap::new(),
            btx: broadcast::channel(256).0,
            atx: mpsc::unbounded_channel().0,
            ptx: mpsc::unbounded_channel().0,
        }))
    }
}
