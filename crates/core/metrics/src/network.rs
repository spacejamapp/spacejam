//! Metrics for the network.

use prometheus as prometheus_client;
use prometheus::{
    encoding::EncodeLabelSet,
    metrics::{family::Family, gauge::Gauge},
    registry::Registry,
};
use std::ops::Deref;

/// Connection metric.
#[derive(Clone, Default)]
pub struct Connection {
    /// The inner metric.
    inner: Family<Peer, Gauge>,
}

impl Connection {
    /// Increment the established connection counter.
    pub fn establish_connection(&self, peer: String) {
        self.inner
            .get_or_create(&Peer { peer })
            .set(Peer::established());
    }

    /// Decrement the connection counter.
    pub fn close_connection(&self, peer: String) {
        self.inner.get_or_create(&Peer { peer }).set(Peer::closed());
    }

    /// Register the metric.
    pub fn register(&self, registry: &mut Registry) {
        registry.register(
            "conn",
            "Connection status, 1: established, 0: closed",
            self.inner.clone(),
        );
    }
}

impl Deref for Connection {
    type Target = Family<Peer, Gauge>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Peer.
#[derive(Default, PartialEq, Eq, Debug, Clone, EncodeLabelSet, Hash)]
pub struct Peer {
    /// Peer ID.
    pub peer: String,
}

impl Peer {
    /// if the target peer is established.
    pub const fn established() -> i64 {
        1
    }

    /// if the target peer is closed.
    pub const fn closed() -> i64 {
        0
    }
}
