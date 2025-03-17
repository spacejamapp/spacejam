//! Events for peers.

use crate::{
    peer::{Connection, PeerId},
    Network,
};
use score::extrinsic::TicketEnvelope;
use std::fmt;

pub mod broadcast;
mod conn;
pub mod sync;

/// Events for peers.
#[derive(Debug, Clone)]
pub enum Event {
    /// Distribute a ticket.
    DistributeTicket {
        /// The epoch.
        epoch: u32,

        /// The ticket.
        ticket: Box<TicketEnvelope>,
    },
    /// A new peer has connected.
    Connected(Connection),
    /// A peer has disconnected.
    Closed {
        /// The peer id
        peer: PeerId,

        /// The reason for the disconnect.
        reason: String,
    },
}

impl Event {
    /// Handle the event.
    pub async fn handle<C: score::runtime::Config>(
        self,
        runtime: Network<C>,
    ) -> anyhow::Result<()> {
        match self {
            Self::DistributeTicket { epoch, ticket } => {
                broadcast::ticket(runtime, epoch, *ticket).await?;
            }
            Self::Connected(conn) => {
                conn::connected(runtime, conn).await;
            }
            Self::Closed { peer, reason } => {
                if let Some(address) = conn::closed(runtime.clone(), peer, reason.clone()).await? {
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    runtime.transport.dial(address).await?;
                }
            }
        }

        Ok(())
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DistributeTicket { epoch, ticket: _ } => {
                write!(f, "DistributeTicket({})", epoch)
            }
            Self::Connected(conn) => {
                write!(f, "Connected({})", conn.address.peer_id)
            }
            Self::Closed { peer, reason: _ } => {
                write!(f, "Closed({})", peer)
            }
        }
    }
}
