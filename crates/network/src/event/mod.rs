//! Events for the network.

pub mod peer;

/// Events for the network.
pub enum Event {
    /// A peer event.
    Peer(peer::Event),
}


