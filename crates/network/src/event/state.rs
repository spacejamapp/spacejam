//! Event handling for the state protocol.

use crate::Network;
use litep2p::protocol::request_response::RequestResponseEvent;

impl Network {
    /// Handle a state event.
    pub fn state(&mut self, event: RequestResponseEvent) {
        tracing::info!("state: {event:?}");
    }
}
