//! Events for peers.

use crate::{stream, Network};

mod conn;

/// Events for peers.
pub enum Event {
    /// Announce a block.
    AnnounceBlock(Vec<u8>),

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
        &self,
        runtime: Network<C>,
    ) -> anyhow::Result<()> {
        match self {
            Self::AnnounceBlock(announcement) => {
                runtime.announce.send(announcement.clone())?;
            }
            Self::RequestBlock { conn, data } => {
                let blocks = stream::ce128::send(conn.clone(), data.clone()).await?;
                for block in blocks {
                    if let Err(e) = runtime.finalize(&block).await {
                        tracing::error!("failed to finalize block#{}: {}", block.header.slot, e);
                        break;
                    }
                }
            }
            Self::Connected {
                peer,
                connection,
                outgoing,
            } => {
                conn::connected(runtime, *peer, connection, *outgoing).await;
            }
            Self::Closed { peer, reason } => {
                conn::closed(runtime, *peer, reason.clone()).await?;
            }
        }

        Ok(())
    }
}
