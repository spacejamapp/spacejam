//! Validating node implementation

use crate::{node::spec::NodeSpec, utils::log};
use network::Network;
use score::{block, extrinsic::ticket};

/// Validating and authoring blocks with network
pub struct Validating<C: runtime::Config>(pub(crate) Network<C>);

impl<C: runtime::Config> Validating<C> {
    /// Authoring service
    #[tracing::instrument(skip_all, name = "author")]
    async fn author(runtime: &Network<C>) {
        log::init(runtime).await;
        let mut author = runtime.author();

        loop {
            tokio::time::sleep(block::next_slot()).await;

            // get the current epoch
            let timeslot = block::timeslot();
            let epoch = timeslot / score::EPOCH_LENGTH;
            let chain = runtime.runtime.chain.read().await;
            if let Ok(best) = chain.best() {
                runtime.dial_validators().await;
                let finalized = chain.grandpa.handshake.head.clone();
                if ticket::subscribe(timeslot % score::EPOCH_LENGTH, best.slot, finalized.slot) {
                    tokio::spawn({
                        let runtime = runtime.clone();
                        async move {
                            if let Err(e) = runtime.subscribe_tickets().await {
                                tracing::error!("Failed to subscribe tickets: {:?}", e);
                            }
                        }
                    });
                }
            };

            // author block and maybe generate ticket
            let (block, ticket) = match author.on_timeslot(timeslot).await {
                Ok((header, ticket)) => (header, ticket),
                Err(e) => {
                    tracing::error!("Authoring error: {:?}", e);
                    continue;
                }
            };

            log::current(runtime).await;

            // author block
            if let Some(block) = block {
                let hash = block.header.hash().expect("failed to get hash");
                tracing::info!("block#{}@0x{}", block.header.slot, hex::encode(&hash[..3]));
                if let Err(e) = runtime.announce(block.header.clone()).await {
                    tracing::error!("Failed to announce block: {:?}", e);
                }

                if let Err(e) = runtime.chain.write().await.import(&block).await {
                    tracing::error!("Failed to import block {e:?}")
                }
            }

            // submit ticket
            if let Some(ticket) = ticket {
                tokio::spawn({
                    let runtime = runtime.clone();
                    async move {
                        if let Err(e) = runtime.submit(epoch, ticket).await {
                            tracing::error!("Failed to submit ticket: {:?}", e);
                        }
                    }
                });
            }

            if let Err(e) = runtime.finalize().await {
                tracing::warn!("Failed to subscribe to hooks: {:?}", e);
            }
        }
    }
}

impl<C: runtime::Config> NodeSpec for Validating<C> {
    async fn start(self) -> anyhow::Result<()> {
        let runtime = self.0.clone();

        tokio::select! {
            _ = Self::author(&runtime) => {}
            _ = runtime.spawn() => {}
            _ = tokio::signal::ctrl_c() => {}
        }

        Ok(())
    }
}
