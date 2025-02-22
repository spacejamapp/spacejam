//! Events for peers.

use crate::Context;
use std::sync::Arc;

/// Events for peers.
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

impl Event {
    /// Handle the event.
    pub fn handle(&self, context: &impl Context) -> anyhow::Result<()> {
        match self {
            Self::ConnectionEstablished { peer } => {
                // context.on_connection_established(peer);
            }
            Self::ConnectionClosed { peer } => {
                // context.on_connection_closed(peer);
            }
        }

        Ok(())
    }
}
