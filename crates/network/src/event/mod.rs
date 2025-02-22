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
    pub fn handle<C: Context>(&self, context: Arc<C>) -> anyhow::Result<()> {
        match self {
            Self::Peer(e) => e.handle(context),
            Self::Action(a) => Ok(()),
        }
    }
}
