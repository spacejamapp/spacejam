//! Events for peers.

use crate::Context;
use std::sync::Arc;

/// Events for peers.
pub enum Event {
    /// A new peer has connected.
    Connected {
        /// The peer's public key.
        peer: [u8; 32],
    },

    /// A peer has disconnected.
    Closed {
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
    pub fn handle<C: Context>(&self, context: Arc<C>) -> anyhow::Result<()> {
        match self {
            Self::Connected { peer } => {
                // context.on_connection_established(peer);
            }
            Self::Closed { peer } => {
                // context.on_connection_closed(peer);
            }
        }

        Ok(())
    }
}
