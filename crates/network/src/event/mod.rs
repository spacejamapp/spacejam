//! Event handling for the network.

use crate::Network;
use litep2p::Litep2pEvent;
use tokio_stream::StreamExt;

mod block;
mod kad;
mod mdns;
mod ping;
mod state;
mod sync;

impl Network {
    /// Spawn the network.
    pub async fn spawn(&mut self) {
        let listen_addresses = self.p2p.listen_addresses().collect::<Vec<_>>();
        tracing::info!("listen addresses: {listen_addresses:?}");

        loop {
            tokio::select! {
                Some(event) = self.block.next() => self.block(event),
                Some(event) = self.sync.next() => self.sync(event),
                Some(event) = self.state.next() => self.state(event),
                Some(event) = self.ping.next() => self.ping(event).await,
                Some(event) = self.kad.next() => self.kad(event).await,
                Some(event) = self.mdns.next() => self.mdns(event).await,
                Some(event) = self.p2p.next_event() => self.litep2p(event).await,
            }
        }
    }

    /// Handle Litep2p events.
    #[tracing::instrument(skip_all, level = "trace")]
    pub async fn litep2p(&mut self, event: Litep2pEvent) {
        match event {
            Litep2pEvent::ConnectionClosed {
                peer,
                connection_id,
            } => {
                tracing::trace!("connection {peer} closed: {connection_id:?}");
            }
            Litep2pEvent::ConnectionEstablished { peer, endpoint } => {
                tracing::trace!("connection {peer} established: {endpoint:?}");
            }
            Litep2pEvent::DialFailure { address, error } => {
                tracing::warn!("dial {address:?} failure: {error:?}");
            }
            Litep2pEvent::ListDialFailures { errors } => {
                tracing::warn!("dial failures: {errors:?}");
            }
        }
    }
}
