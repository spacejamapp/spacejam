//! Events for the network.

use crate::Network;

pub mod action;
pub mod conn;

/// Events for the network.
pub enum Event {
    /// A peer event.
    Peer(conn::Event),

    /// An action.
    Action(action::Event),
}

impl Event {
    /// Handle the event.
    pub async fn handle<C: score::runtime::Config>(
        &self,
        runtime: Network<C>,
    ) -> anyhow::Result<()> {
        match self {
            Self::Peer(e) => e.handle(runtime).await,
            Self::Action(a) => a.handle(runtime).await,
        }
    }
}
