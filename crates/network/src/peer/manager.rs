//! Peer manager.

use std::collections::{BTreeMap, HashMap};

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
}
