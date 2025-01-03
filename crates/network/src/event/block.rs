//! Event handling for the block announceprotocol.

use crate::Network;
use litep2p::protocol::notification::NotificationEvent;

impl Network {
    /// Handle a block announce event.
    pub fn block(&mut self, event: NotificationEvent) {
        tracing::info!("block announce: {event:?}");
    }
}
