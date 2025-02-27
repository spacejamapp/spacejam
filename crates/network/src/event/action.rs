//! Internal actions for the network.

use crate::{context::Context, Network};

/// Internal actions for the network.
///
/// This is a special event in the network which is only for internal use.
pub enum Event {
    /// Announce a block.
    AnnounceBlock(Vec<u8>),
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
        context: Network<C>,
    ) -> anyhow::Result<()> {
        let tx = context.manager.read().await.btx.clone();

        match self {
            Self::AnnounceBlock(announce) => {
                let count = tx.send(announce.clone())?;
                tracing::trace!("Announced block to {count} peers");
                Ok(())
            }
        }
    }
}
