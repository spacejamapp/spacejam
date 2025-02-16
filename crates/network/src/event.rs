//! Event for the transport.

/// Event for the transport.
pub enum Event {
    /// Connection established.
    ConnectionEstablished { peer: [u8; 32] },

    /// Connection closed.
    ConnectionClosed { peer: [u8; 32] },
}
