//! Development node implementation

use crate::node::spec::NodeSpec;
use runtime::Runtime;
use score::block;
use std::time::Duration;

/// Authoring blocks per 6 secs without network
pub struct Dev<C: runtime::Config>(pub(crate) Runtime<C>);

impl<C: runtime::Config> NodeSpec for Dev<C> {
    async fn start(self) -> anyhow::Result<()> {
        let author = self.0.author();
        loop {
            let now = block::now();
            let duration = (score::SLOT_PERIOD - (now % score::SLOT_PERIOD)) as u64;
            tokio::time::sleep(Duration::from_secs(duration)).await;

            let timeslot = block::timeslot();
            let header = author.author(timeslot).await?;
            tracing::info!(
                "block#{}@0x{}",
                header.slot,
                hex::encode(&header.hash()?[..3])
            );
        }
    }
}
