//! Events for peers.

use crate::{
    peer::{Connection, PeerId},
    Network,
};
use score::{block::Header, extrinsic::TicketEnvelope, runtime::Head, TimeSlot};
use std::fmt;

mod broadcast;
mod conn;
mod sync;

/// Events for peers.
#[derive(Debug, Clone)]
pub enum Event {
    /// Announce a block.
    AnnounceBlock {
        /// The block.
        header: Box<Header>,

        /// The head.
        head: Head,
    },
    /// Distribute a ticket.
    DistributeTicket {
        /// The epoch.
        epoch: u32,

        /// The ticket.
        ticket: Box<TicketEnvelope>,
    },
    /// Select the best chain.
    SelectBestChain { slot: TimeSlot },
    /// A new peer has connected.
    Connected {
        /// The connection.
        conn: Connection,
    },
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
        tracing::trace!("handling event: {self}");
        match self {
            Self::AnnounceBlock { header, head } => {
                broadcast::announce(runtime, header, head).await?;
            }
            Self::DistributeTicket { epoch, ticket } => {
                broadcast::ticket(runtime, epoch, *ticket).await?;
            }
            Self::Connected { conn } => {
                conn::connected(runtime, conn).await;
            }
            Self::Closed { peer, reason } => {
                if let Some(address) = conn::closed(runtime.clone(), peer, reason.clone()).await? {
                    runtime.transport.dial(address).await?;
                }
            }
            Self::SelectBestChain { slot } => {
                sync::select_best_chain(runtime, slot).await?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AnnounceBlock { header, head: _ } => {
                write!(f, "AnnounceBlock({})", header.slot,)
            }
            Self::DistributeTicket { epoch, ticket: _ } => {
                write!(f, "DistributeTicket({})", epoch)
            }
            Self::SelectBestChain { slot } => {
                write!(f, "SelectBestChain({})", slot)
            }
            Self::Connected { conn } => {
                write!(f, "Connected({})", conn.address.peer_id)
            }
            Self::Closed { peer, reason: _ } => {
                write!(f, "Closed({})", peer)
            }
        }
    }
}
