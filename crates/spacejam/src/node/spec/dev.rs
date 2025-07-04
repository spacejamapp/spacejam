//! Development node implementation

use crate::node::spec::NodeSpec;
use offchain::Offchain;
use runtime::{Runtime, Validator};
use score::block;
use std::{net::SocketAddr, sync::Arc};

/// Authoring blocks per 6 secs without network
pub struct Dev<C: runtime::Config> {
    pub(crate) runtime: Runtime<C>,
    pub(crate) rpc: SocketAddr,
}

impl<C: runtime::Config> Dev<C> {
    /// Authoring blocks per 6 secs without network
    async fn author(runtime: Arc<Runtime<C>>) -> anyhow::Result<()> {
        let author = runtime.author();
        loop {
            tokio::time::sleep(block::next_slot()).await;

            let timeslot = block::timeslot();
            let block = author.author(timeslot).await?;
            tracing::info!(
                "block#{}@0x{}",
                block.header.slot,
                hex::encode(&block.header.hash()?[..3])
            );

            let mut chain = runtime.chain_mut().await;
            chain.import(&block)?;
        }
    }
}

impl<C: runtime::Config> NodeSpec for Dev<C> {
    async fn start(mut self) -> anyhow::Result<()> {
        tracing::info!("Running spacejam in dev mode");
        tracing::debug!("development seed: 0x{}", hex::encode([0; 32]));
        self.runtime.validator = C::Validator::dev();
        let runtime = Arc::new(self.runtime);
        let offchain = Offchain::new(runtime.clone());

        tokio::select! {
            r = Self::author(runtime) => r,
            r = offchain.start(self.rpc) => r,
            _ = tokio::signal::ctrl_c() => Ok(())
        }
    }
}
