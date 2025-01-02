//! Event handling for the network.

use crate::Network;
use litep2p::{Litep2p, Litep2pEvent};
use std::rc::Rc;
use tokio::sync::RwLock;
use tokio_stream::StreamExt;

mod block;
mod ping;
mod state;
mod sync;

impl Network {
    /// Start the network.
    pub async fn spawn_litep2p(litep2p: Rc<RwLock<Litep2p>>) {
        while let Some(event) = litep2p.write().await.next_event().await {
            match event {
                Litep2pEvent::ConnectionClosed {
                    peer,
                    connection_id,
                } => {
                    // TODO: remove the peer from the peer manager.
                    tracing::trace!("connection {peer} closed: {connection_id:?}");
                }
                Litep2pEvent::ConnectionEstablished { peer, endpoint } => {
                    // TODO: add the peer to the peer manager.
                    tracing::trace!("connection {peer} established: {endpoint:?}");
                }
                Litep2pEvent::DialFailure { address, error } => {
                    // TODO: remove the peer from the peer manager.
                    tracing::warn!("dial {address:?} failure: {error:?}");
                }
                Litep2pEvent::ListDialFailures { errors } => {
                    // TODO: remove the peers from the peer manager.
                    tracing::warn!("dial failures: {errors:?}");
                }
            }
        }
    }

    /// Spawn the event handler.
    pub async fn spawn_events(&mut self) {
        loop {
            tokio::select! {
                Some(event) = self.block.next() => self.block(event),
                Some(event) = self.sync.next() => self.sync(event),
                Some(event) = self.state.next() => self.state(event),
                Some(event) = self.ping.next() => self.ping(event).await,
            }
        }
    }
}
