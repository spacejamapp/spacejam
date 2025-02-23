//! Events for peers.

use crate::{peer::Address, Context};
use std::sync::Arc;

/// Events for peers.
pub enum Event {
    /// A new peer has connected.
    Connected {
        /// The peer's public key.
        address: Address,
    },

    /// A peer has disconnected.
    Closed {
        /// The peer's public key.
        address: Address,
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
            Self::Connected { address } => {
                tracing::debug!("connected to {}", address);
                context
                    .metrics()
                    .conn
                    .establish_connection(address.to_string());
            }
            Self::Closed { address } => {
                tracing::debug!("disconnected from {}", address);
                context.metrics().conn.close_connection(address.to_string());
            }
        }

        Ok(())
    }
}
