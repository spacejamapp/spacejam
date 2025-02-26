//! Internal actions for the network.

use crate::{context::Context, peer::Manager};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Internal actions for the network.
///
/// This is a special event in the network which is only for internal use.
pub enum Event {
    /// Announce a block.
    AnnounceBlock(Vec<u8>),

    /// Request a block.
    RequestBlock,
}

impl From<Event> for crate::Event {
    fn from(event: Event) -> Self {
        crate::Event::Action(event)
    }
}

impl Event {
    /// Handle the action event.
    pub async fn handle<C: Context + Send + Sync + 'static>(
        &self,
        _context: Arc<C>,
        manager: Arc<RwLock<Manager>>,
    ) -> anyhow::Result<()> {
        match self {
            Self::AnnounceBlock(announce) => {
                let count = manager.write().await.btx.send(announce.clone())?;
                tracing::trace!("Announced block to {count} peers");
                Ok(())
            }
            Self::RequestBlock => Ok(()),
        }
    }
}
