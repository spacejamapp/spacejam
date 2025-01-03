//! Event handling for the ping protocol.

use crate::Network;
use litep2p::protocol::libp2p::ping::PingEvent;

impl Network {
    /// Handle a ping event.
    ///
    /// TODO: remove slow peers from the peer manager.
    #[tracing::instrument(skip_all, level = "trace")]
    pub async fn ping(&mut self, event: PingEvent) {
        let PingEvent::Ping { peer, ping } = event;
        tracing::trace!("ping from {peer:?}: {ping:?}");
    }
}
