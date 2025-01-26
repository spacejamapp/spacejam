//! Event handling for the block announceprotocol.

use crate::Network;
use litep2p::protocol::notification::NotificationEvent;

impl Network {
    /// Handle a block announce event.
    pub fn block(&mut self, event: NotificationEvent) {
        tracing::info!("block announce: {event:?}");
        if let NotificationEvent::NotificationReceived {
            peer: _,
            notification,
        } = event
        {
            if let Err(e) = self.context.import_block(notification.to_vec()) {
                tracing::error!("failed to import block: {e}");
            }
        }
    }
}
