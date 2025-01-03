//! Event handling for the ping protocol.

use crate::Network;
use litep2p::protocol::libp2p::ping::PingEvent;

impl Network {
    /// Handle a ping event.
    pub async fn ping(&mut self, event: PingEvent) {
        let PingEvent::Ping { peer, ping } = event;
        tracing::trace!("ping from {peer:?}: {ping:?}");

        
        if let Err(e) = self.p2p.dial(&peer).await {
            tracing::warn!("dial {peer:?} failure: {e:?}");
        }
    }
}
