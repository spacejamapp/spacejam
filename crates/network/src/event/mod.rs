//! Event handling for the network.

use crate::{Context, Network};
use litep2p::Litep2pEvent;
use tokio_stream::StreamExt;

mod block;
mod kad;
mod mdns;
mod ping;
mod state;
mod sync;

/// Event handling for the network.
pub enum Event {
    /// Subscribe a new block.
    SubscribeBlock(Vec<u8>),

    /// Subscribe a new ticket.
    SubscribeTicket(Vec<u8>),
}

impl Network {
    /// Handle Spacejam events.
    pub async fn spacejam(&mut self, context: &impl Context) {
        while let Some(event) = self.rx.recv().await {
            match event {
                Event::SubscribeBlock(block) => {
                    if let Err(e) = context.subscribe_block(block) {
                        tracing::error!("failed to subscribe to block: {e}");
                    }
                }
                Event::SubscribeTicket(_ticket) => {
                    // TODO: subscribe ticket to the network
                }
            }
        }
    }

    /// Handle Litep2p events.
    #[tracing::instrument(skip_all, level = "trace")]
    pub async fn litep2p(&mut self, event: Litep2pEvent, context: &impl Context) {
        match event {
            Litep2pEvent::ConnectionClosed {
                peer,
                connection_id,
            } => {
                tracing::trace!("connection {peer} closed: {connection_id:?}");
                context.metrics().conn.close_connection(peer.to_string());
                self.peer.remove(peer, connection_id);
            }
            Litep2pEvent::ConnectionEstablished { peer, endpoint } => {
                tracing::trace!("connection {peer} established: {endpoint:?}");
                context
                    .metrics()
                    .conn
                    .establish_connection(peer.to_string());
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

    /// Spawn the network.
    pub async fn spawn(&mut self, context: &impl Context) {
        loop {
            tokio::select! {
                Some(event) = self.block.next() => self.block(event, context),
                Some(event) = self.sync.next() => self.sync(event),
                Some(event) = self.state.next() => self.state(event),
                Some(event) = self.ping.next() => self.ping(event).await,
                Some(event) = self.kad.next() => self.kad(event).await,
                Some(event) = self.mdns.next() => self.mdns(event).await,
                Some(event) = self.p2p.next_event() => self.litep2p(event, context).await,
            }
        }
    }
}
