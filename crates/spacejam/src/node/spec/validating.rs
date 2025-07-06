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
        let mut author = runtime.author();

        loop {
            tokio::time::sleep(block::next_slot()).await;
            tracing::trace!("sleep done, new timeslot");

            // get the current epoch
            let timeslot = block::timeslot();
            let epoch = timeslot / score::EPOCH_LENGTH;

            tracing::trace!("try to dial validators");
            if let Ok(best) = runtime.best().await {
                runtime.dial_validators().await;
                let finalized = runtime.finalized().await;
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
            tracing::trace!("try authoring block and ticket");
            let (block, ticket) = match author.on_timeslot(timeslot).await {
                Ok((header, ticket)) => (header, ticket),
                Err(e) => {
                    tracing::error!("Authoring error: {:?}", e);
                    continue;
                }
            };

            tracing::trace!("authoring block and ticket done");
            log::current(runtime).await;

            // author block
            if block::timeslot() == timeslot {
                tracing::trace!("check authored block ...");
                if let Some(block) = block {
                    let hash = block.header.hash().expect("failed to get hash");
                    tracing::info!("block#{}@0x{}", block.header.slot, hex::encode(&hash[..3]));
                    if let Err(e) = runtime.announce(block.header.clone()).await {
                        tracing::error!("Failed to announce block: {:?}", e);
                    }

                    tracing::trace!(
                        "try acquiring the chain write lock for importing authored block"
                    );
                    if let Err(e) = runtime.chain_mut().await.import(&block) {
                        tracing::error!("Failed to import block {e:?}")
                    }
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
