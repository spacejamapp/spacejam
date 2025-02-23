//! Events for the network.

use crate::Context;
pub use action::Action;
use std::sync::Arc;

pub mod action;
pub mod peer;

/// Events for the network.
pub enum Event {
    /// A peer event.
    Peer(peer::Event),

    /// An action.
    Action(Action),
}

impl Event {
    /// Handle the event.
    pub fn handle<C: Context>(&self, context: Arc<C>) -> anyhow::Result<()> {
        match self {
            Self::Peer(e) => e.handle(context),
            Self::Action(_a) => Ok(()),
        }
    }

    /// Handle the event without checking for errors.
    pub fn handle_unchecked<C: Context>(&self, context: Arc<C>) {
        if let Err(e) = self.handle(context) {
            tracing::error!("{e:?}");
        }
    }
}
