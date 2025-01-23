use prometheus as prometheus_client;
use prometheus::encoding::EncodeLabelSet;

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
