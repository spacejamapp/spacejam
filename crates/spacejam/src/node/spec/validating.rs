//! Validating node implementation

use crate::{node::spec::NodeSpec, utils::log};
use network::Network;
use runtime::storage::SyncStorage;
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
            if let Ok(best) = runtime.chain.read().await.best() {
                runtime.dial_validators().await;
                let Ok(finalized) = runtime.storage.finalized() else {
                    tracing::error!("Failed to get finalized block");
                    continue;
                };

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
            let (header, ticket) = match author.on_timeslot(timeslot).await {
                Ok((header, ticket)) => (header, ticket),
                Err(e) => {
                    tracing::error!("Authoring error: {:?}", e);
                    continue;
                }
            };

            log::current(runtime).await;

            // author block
            if let Some(header) = header {
                if let Ok(hash) = header.hash() {
                    let Ok(parent) = runtime.storage.block(&header.parent) else {
                        tracing::error!(
                            "Failed to get parent header of authored block#{}",
                            header.slot
                        );
                        continue;
                    };

                    tracing::info!(
                        "block#{}@0x{}, parent#{}@0x{}",
                        header.slot,
                        hex::encode(&hash[..3]),
                        parent.header.slot,
                        hex::encode(&header.parent[..3])
                    );
                }

                if let Err(e) = runtime.announce(header.clone()).await {
                    tracing::error!("Failed to announce block: {:?}", e);
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
