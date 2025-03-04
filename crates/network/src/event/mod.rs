//! Events for peers.

use crate::Network;

mod peer;

/// Events for peers.
pub enum Event {
    /// Announce a block.
    AnnounceBlock(Vec<u8>),

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
            Self::Connected {
                peer,
                connection,
                outgoing,
            } => {
                peer::connected(runtime, *peer, connection, *outgoing).await;
            }
            Self::Closed { peer, reason } => {
                peer::closed(runtime, *peer, reason.clone()).await?;
            }
        }

        Ok(())
    }
}
