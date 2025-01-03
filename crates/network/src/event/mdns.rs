//! mDNS event handling.

use crate::Network;
use litep2p::protocol::mdns::MdnsEvent;

impl Network {
    /// Handle an mDNS event.
    pub async fn mdns(&self, event: MdnsEvent) {
        let MdnsEvent::Discovered(addresses) = event;

        for address in addresses {
            if self.address_exists(&address).await {
                continue;
            }

            tracing::info!("dialing {address:?}");

            // records an event outside of any span context:
            tracing::event!(tracing::Level::INFO, "something happened");

            let span = tracing::span!(tracing::Level::INFO, "my_span");
            let _guard = span.enter();

            // records an event within "my_span".
            tracing::event!(tracing::Level::DEBUG, "dialing {address:?}");

            // tracing::debug!("dialing {address:?}");
            if let Err(e) = self.p2p.write().await.dial_address(address.clone()).await {
                tracing::warn!("failed to dial {address:?}: {e:?}");
            }
        }
    }
}
