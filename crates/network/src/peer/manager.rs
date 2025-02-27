//! Peer manager.

use crate::event::Event;
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

    /// Action sender.
    pub tx: mpsc::UnboundedSender<Event>,
}

impl Manager {
    /// Create a new manager.
    pub fn new(tx: mpsc::UnboundedSender<Event>) -> Self {
        Self {
            conns: HashMap::new(),
            btx: broadcast::channel(256).0,
            tx,
        }
    }

    /// Insert a new connection.
    pub fn insert(&mut self, peer: [u8; 32], conn: quinn::Connection) {
        self.conns.insert(peer, conn);
    }
}
