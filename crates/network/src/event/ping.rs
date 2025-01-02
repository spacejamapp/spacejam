//! Event handling for the ping protocol.

use crate::Network;
use litep2p::protocol::libp2p::ping::PingEvent;

impl Network {
    /// Handle a ping event.
    pub fn ping(&self, event: PingEvent) {
        tracing::info!("ping: {event:?}");
    }
}
