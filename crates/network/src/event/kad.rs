//! Kademlia event handling.

use crate::Network;
use litep2p::protocol::libp2p::kademlia::KademliaEvent;

impl Network {
    /// Handle a Kademlia event.
    pub async fn kad(&self, event: KademliaEvent) {
        println!("Kademlia event: {event:?}");
    }
}
