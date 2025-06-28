//! Validating node implementation

use crate::{node::spec::NodeSpec, utils::log};
use network::Network;
use runtime::storage::SyncStorage;
use score::block;
use std::time::Duration;

/// Validating and authoring blocks with network
pub struct Validating<C: runtime::Config>(pub(crate) Network<C>);

impl<C: runtime::Config> Validating<C> {
    /// Authoring service
    #[tracing::instrument(skip_all, name = "author")]
    async fn author(runtime: &Network<C>) {
        log::init(runtime).await;
        let mut author = runtime.author();
        let Ok(best) = runtime.storage.best() else {
            tracing::error!("Failed to get best block");
            return;
        };
        let epoch = best.slot / score::EPOCH_LENGTH;
        if let Err(e) = author.on_new_epoch(epoch).await {
            tracing::error!("Failed to initialize authoring: {e:?}");
            return;
        }

        loop {
            tokio::time::sleep(block::next_slot()).await;
            let Ok(best) = runtime.storage.best() else {
                tracing::error!("Failed to get best block");
                tokio::time::sleep(Duration::from_secs(score::SLOT_PERIOD as u64)).await;
                continue;
            };

            // dial lost connections
            if best.slot != 0 && best.slot % score::EPOCH_LENGTH > 1 {
                runtime.dial_validators().await;
            }

            // get the current epoch
            log::current(runtime).await;
            let timeslot = block::timeslot();
            let epoch = timeslot / score::EPOCH_LENGTH;
            let prev = timeslot.saturating_sub(1);
            if best.slot < prev {
                // select the best chain before authoring
                if let Err(e) = runtime.select_best_chain(timeslot).await {
                    tracing::error!("Failed to select best chain: {:?}", e);
                    continue;
                }
            }

            // author block and maybe generate ticket
            let (header, ticket) = match author.on_timeslot(timeslot).await {
                Ok((header, ticket)) => (header, ticket),
                Err(e) => {
                    tracing::error!("Authoring error: {:?}", e);
                    continue;
                }
            };

            // check subscribing tickets
            {
                let Ok(finalized) = runtime.storage.finalized() else {
                    tracing::error!("Failed to get finalized block");
                    continue;
                };

                if best.subscribe_tickets(timeslot, finalized.slot) {
                    tokio::spawn({
                        let runtime = runtime.clone();
                        async move {
                            if let Err(e) = runtime.subscribe_tickets().await {
                                tracing::error!("Failed to subscribe tickets: {:?}", e);
                            }
                        }
                    });
                }
            }

            // send ticket
            if let Some(ticket) = ticket {
                tokio::spawn({
                    let runtime = runtime.clone();
                    async move {
                        if let Err(e) = runtime.submit(epoch, ticket).await {
                            tracing::error!("Failed to send ticket: {:?}", e);
                        }
                    }
                });
            }

            // author block
            if let Some(header) = header {
                if let Ok(hash) = header.hash() {
                    tracing::info!(
                        "block#{}@0x{}, parent@{}",
                        header.slot,
                        hex::encode(&hash[..3]),
                        hex::encode(&header.parent[..3])
                    );
                }

                if let Err(e) = runtime.announce(Box::new(header.clone())).await {
                    tracing::error!("Failed to announce block: {:?}", e);
                }
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
