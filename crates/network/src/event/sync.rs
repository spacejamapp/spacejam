//! Event handling for the sync protocol.

use crate::Network;
use litep2p::protocol::request_response::RequestResponseEvent;

impl Network {
    /// Handle a sync event.
    pub fn sync(&mut self, event: RequestResponseEvent) {
        tracing::info!("sync: {event:?}");
    }
}
