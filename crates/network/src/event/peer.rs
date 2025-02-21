//! Events for peers.

pub enum Event {
    /// A new peer has connected.
    ConnectionEstablished {
        /// The peer's public key.
        peer: [u8; 32],
    },

    /// A peer has disconnected.
    ConnectionClosed {
        /// The peer's public key.
        peer: [u8; 32],
    },
}

impl From<Event> for crate::Event {
    fn from(event: Event) -> Self {
        crate::Event::Peer(event)
    }
}
