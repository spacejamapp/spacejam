//! Internal actions for the network.

use crate::context::Context;
use std::sync::Arc;

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
        context: Arc<C>,
    ) -> anyhow::Result<()> {
        let manager = context.manager();

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
