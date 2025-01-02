//! Event handling for the network.

use litep2p::{
    protocol::{
        libp2p::ping::PingEvent, notification::NotificationEvent,
        request_response::RequestResponseEvent,
    },
    Litep2pEvent,
};

use crate::Network;

mod block;
mod ping;
mod state;
mod sync;

/// Event type.
#[derive(Debug)]
pub enum Event {
    /// Block event.
    Block(NotificationEvent),
    /// Sync event.
    Sync(RequestResponseEvent),
    /// State event.
    State(RequestResponseEvent),
    /// Ping event.
    Ping(PingEvent),
}

impl Network {
    /// Start the network.
    pub async fn spawn_litep2p(&self) {
        while let Some(event) = self.p2p.write().await.next_event().await {
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

    /// Spawn the event handler.
    pub async fn spawn_events(&self) {
        while let Some(event) = self.rx.lock().await.recv().await {
            match event {
                Event::Block(event) => self.block(event),
                Event::Sync(event) => self.sync(event),
                Event::State(event) => self.state(event),
                Event::Ping(event) => self.ping(event),
            }
        }
    }
}
