//! Events for the network.

use crate::{peer::Manager, Context};
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod action;
pub mod peer;

/// Events for the network.
pub enum Event {
    /// A peer event.
    Peer(peer::Event),

    /// An action.
    Action(action::Event),
}

impl Event {
    /// Handle the event.
    pub async fn handle<C: Context + Send + Sync + 'static>(
        &self,
        context: Arc<C>,
        manager: Arc<RwLock<Manager>>,
    ) -> anyhow::Result<()> {
        match self {
            Self::Peer(e) => e.handle(context, manager).await,
            Self::Action(a) => a.handle(context, manager).await,
        }
    }

    /// Handle the event without checking for errors.
    pub async fn handle_unchecked<C: Context + Send + Sync + 'static>(
        &self,
        context: Arc<C>,
        manager: Arc<RwLock<Manager>>,
    ) {
        if let Err(e) = self.handle(context, manager).await {
            tracing::error!("{e:?}");
        }
    }
}
