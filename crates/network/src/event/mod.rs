//! Events for the network.

use crate::Context;
use std::sync::Arc;

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
    pub async fn handle<C: Context + Send + Sync + 'static>(
        &self,
        context: Arc<C>,
    ) -> anyhow::Result<()> {
        match self {
            Self::Peer(e) => e.handle(context).await,
            Self::Action(a) => a.handle(context).await,
        }
    }

    /// Handle the event without checking for errors.
    pub async fn handle_unchecked<C: Context + Send + Sync + 'static>(&self, context: Arc<C>) {
        if let Err(e) = self.handle(context).await {
            tracing::error!("{e:?}");
        }
    }
}
