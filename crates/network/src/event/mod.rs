//! Events for peers.

use crate::{stream, Network};

mod conn;
mod request;

/// Events for peers.
pub enum Event {
    /// Announce a block.
    AnnounceBlock(Vec<u8>),

    /// Request blocks.
    RequestBlock {
        /// The peer's public key.
        peer: [u8; 32],

        /// The connection.
        conn: quinn::Connection,

        /// The data.
        data: stream::ce128::Request,

        /// Whether finalize the block on receiving.
        finalize: bool,
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
            Self::RequestBlock {
                peer,
                conn,
                data,
                finalize,
            } => {
                request::blocks(
                    runtime.clone(),
                    *peer,
                    conn.clone(),
                    data.clone(),
                    *finalize,
                )
                .await?;
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
