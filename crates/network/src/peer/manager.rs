//! Peer manager.

use crate::event::peer;
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};

/// Peer manager
pub struct Manager {
    /// Peer connections.
    ///
    /// Introduce network grid here.
    pub conns: HashMap<[u8; 32], quinn::Connection>,

    /// Block announcement sender.
    pub btx: broadcast::Sender<Vec<u8>>,

    /// Peer event sender.
    pub ptx: mpsc::UnboundedSender<peer::Event>,
}

impl Manager {
    /// Create a new manager.
    pub fn new(btx: broadcast::Sender<Vec<u8>>, ptx: mpsc::UnboundedSender<peer::Event>) -> Self {
        Self {
            conns: HashMap::new(),
            btx,
            ptx,
        }
    }

    /// Insert a new connection.
    pub fn insert(&mut self, peer: [u8; 32], conn: quinn::Connection) {
        self.conns.insert(peer, conn);
    }
}
