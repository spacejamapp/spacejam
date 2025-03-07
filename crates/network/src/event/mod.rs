//! Events for peers.

use crate::{peer::PeerId, Network};
use score::{block::Header, runtime::Head, TimeSlot};

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
    /// Select the best chain.
    SelectBestChain { slot: TimeSlot },

    /// A new peer has connected.
    Connected {
        /// The peer's public key.
        peer: PeerId,

        /// The connection.
        connection: quinn::Connection,

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
            Self::AnnounceBlock { header, head } => {
                sync::announce(runtime, header, head).await?;
            }
            Self::SelectBestChain { slot } => {
                sync::select_best_chain(runtime, slot).await?;
            }
            Self::Connected {
                peer,
                connection,
                outgoing,
            } => {
                conn::connected(runtime, peer, &connection, outgoing).await;
            }
            Self::Closed { peer, reason } => {
                conn::closed(runtime, peer, reason.clone()).await?;
            }
        }

        Ok(())
    }
}
