//! mDNS event handling.

use crate::Network;
use litep2p::{protocol::mdns::MdnsEvent, PeerId};
use std::time::Duration;

impl Network {
    /// Handle an mDNS event.
    #[tracing::instrument(skip_all, level = "trace")]
    pub async fn mdns(&mut self, event: MdnsEvent) {
        let MdnsEvent::Discovered(addresses) = event;
        tracing::trace!("discovered {addresses:?}");

        for address in addresses.clone() {
            if let Some(peer) = PeerId::try_from_multiaddr(&address) {
                if self.peer.exists(&peer) {
                    tracing::trace!("peer {peer} already exists");
                    continue;
                }
            }

            tracing::trace!("dialing peer {address:?}");
            if let Err(e) = self.p2p.dial_address(address.clone()).await {
                tracing::warn!("failed to dial {address:?}: {e:?}");
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
