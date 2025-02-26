//! Peer manager.

use std::collections::{BTreeMap, HashMap};

use tokio::sync::mpsc;

use crate::event::peer;

/// Peer manager
pub struct Manager {
    /// Peer connections.
    ///
    /// Introduce network grid here.
    pub conns: HashMap<[u8; 32], quinn::Connection>,

    /// Streams.
    ///
    /// UP stream registry some they can only exist one
    /// at the same time.
    pub streams: HashMap<[u8; 32], BTreeMap<u8, bool>>,

    /// Peer event sender.
    pub ptx: mpsc::UnboundedSender<peer::Event>,
}

impl Manager {
    /// Create a new manager.
    pub fn new(ptx: mpsc::UnboundedSender<peer::Event>) -> Self {
        Self {
            conns: HashMap::new(),
            streams: HashMap::new(),
            ptx,
        }
    }
}
