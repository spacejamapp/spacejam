//! Peer management.

use litep2p::{
    transport::Endpoint,
    types::{multiaddr::Multiaddr, ConnectionId},
    PeerId,
};
use std::collections::HashMap;

/// Peer manager.
#[derive(Default, Clone, Debug)]
pub struct PeerManager {
    /// peer addresses.
    addrs: HashMap<PeerId, Vec<Multiaddr>>,

    /// connected peers.
    conns: HashMap<PeerId, ConnectionId>,
}

impl PeerManager {
    /// Add an endpoint.
    pub fn add(&mut self, peer: PeerId, endpoint: Endpoint) {
        self.conns.insert(peer, endpoint.connection_id());
        self.addrs
            .entry(peer)
            .or_default()
            .push(endpoint.address().clone());
    }

    /// Remove a peer.
    pub fn remove(&mut self, peer: PeerId, id: ConnectionId) {
        if self.conns.get(&peer).copied() == Some(id) {
            self.conns.remove(&peer);
            self.addrs.remove(&peer);
        }
    }

    /// Check if a peer exists.
    pub fn exists(&self, peer: &PeerId) -> bool {
        self.conns.contains_key(peer)
    }
}
