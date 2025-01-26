//! Event handling for the network.

use crate::Network;
use litep2p::Litep2pEvent;

mod block;
mod kad;
mod mdns;
mod ping;
mod state;
mod sync;

impl Network {
    /// Handle Litep2p events.
    #[tracing::instrument(skip_all, level = "trace")]
    pub async fn litep2p(&mut self, event: Litep2pEvent) {
        match event {
            Litep2pEvent::ConnectionClosed {
                peer,
                connection_id,
            } => {
                tracing::trace!("connection {peer} closed: {connection_id:?}");
                self.metrics.conn.close_connection(peer.to_string());
                self.peer.remove(peer, connection_id);
            }
            Litep2pEvent::ConnectionEstablished { peer, endpoint } => {
                tracing::trace!("connection {peer} established: {endpoint:?}");
                self.metrics.conn.establish_connection(peer.to_string());
                self.peer.add(peer, endpoint);
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
