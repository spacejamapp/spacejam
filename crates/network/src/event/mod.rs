//! Events for peers.

use crate::{
    peer::{Connection, PeerId},
    Network,
};
use score::{block::Header, extrinsic::TicketEnvelope, runtime::Head, TimeSlot};

mod broadcast;
mod conn;
mod sync;

/// Events for peers.
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

        /// Whether the connection is outgoing.
        outgoing: bool,
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
        match self {
            Self::DistributeTicket { epoch, ticket } => {
                broadcast::ticket(runtime, epoch, *ticket).await?;
            }
            Self::Connected { conn, outgoing } => {
                conn::connected(runtime, conn, outgoing).await;
            }
            Self::Closed { peer, reason } => {
                conn::closed(runtime, peer, reason.clone()).await?;
            }
            Self::AnnounceBlock { header, head } => {
                sync::announce(runtime, header, head).await?;
            }

            Self::SelectBestChain { slot } => {
                sync::select_best_chain(runtime, slot).await?;
            }
        }

        Ok(())
    }
}
