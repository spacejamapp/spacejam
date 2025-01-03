//! Peer management.

use crate::Network;
use litep2p::types::multiaddr::Multiaddr;

impl Network {
    /// Check if a peer exists.
    pub async fn address_exists(&self, address: &Multiaddr) -> bool {
        self.p2p
            .read()
            .await
            .public_addresses()
            .get_addresses()
            .iter()
            .any(|a| a.to_string() == address.to_string())
    }
}
