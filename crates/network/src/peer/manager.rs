//! Peer manager.

use std::collections::HashMap;
use tokio::sync::broadcast;

/// Peer manager
pub struct Manager {
    /// Peer connections.
    ///
    /// Introduce network grid here.
    pub conns: HashMap<[u8; 32], quinn::Connection>,

    /// Block announcement sender.
    pub btx: broadcast::Sender<Vec<u8>>,
}

impl Manager {
    /// Create a new manager.
    pub fn new() -> Self {
        Self {
            conns: Default::default(),
            btx: broadcast::channel(256).0,
        }
    }

    /// Insert a new connection.
    pub fn insert(&mut self, peer: [u8; 32], conn: quinn::Connection) {
        self.conns.insert(peer, conn);
    }
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}
