//! Events for peers.

use crate::{stream, Network};
use score::{block::Header, runtime::Head, TimeSlot};

mod conn;
mod request;
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
    /// Request blocks.
    RequestBlock {
        /// The connection.
        conn: quinn::Connection,

        /// The data.
        data: stream::ce128::Request,
    },
    /// A new peer has connected.
    Connected {
        /// The peer's public key.
        peer: [u8; 32],

        /// The connection.
        connection: quinn::Connection,

        /// Whether the connection is outgoing.
        outgoing: bool,
    },
    /// A peer has disconnected.
    Closed {
        /// The peer id
        peer: [u8; 32],

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
            Self::RequestBlock { conn, data } => {
                request::blocks(runtime.clone(), conn.clone(), data.clone()).await?;
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
