//! Validating node implementation

use crate::node::spec::NodeSpec;
use crate::utils::log;
use network::Network;
use runtime::storage::Storage;
use score::{block, safrole::ValidatorIter};
use std::time::Duration;

/// Validating and authoring blocks with network
pub struct Validating<C: runtime::Config>(pub(crate) Network<C>);

impl<C: runtime::Config> Validating<C> {
    /// Authoring service
    #[tracing::instrument(skip_all, name = "author")]
    async fn author(runtime: &Network<C>) {
        log::init(runtime).await;
        let mut author = runtime.author();
        if let Err(e) = author.on_new_epoch().await {
            tracing::error!("Failed to initialize authoring: {e:?}");
            return;
        }

        loop {
            let now = block::now();
            if !author
                .storage
                .current_validators()
                .unwrap_or_default()
                .bandersnatch()
                .contains(&author.me())
            {
                tracing::warn!("Not in the validator set, sleeping...");
                tokio::time::sleep(Duration::from_secs(
                    ((score::SLOT_PERIOD as u32) * score::EPOCH_LENGTH
                        - now % ((score::SLOT_PERIOD as u32) * score::EPOCH_LENGTH))
                        as u64,
                ))
                .await;
                continue;
            }

            // dial lost connections
            let handshake = runtime.grandpa.read().await.handshake.clone();
            if handshake.head.slot != 0 && handshake.head.slot % score::EPOCH_LENGTH > 1 {
                runtime.dial_validators().await;
            }

            // sleep until the next slot
            let duration =
                ((score::SLOT_PERIOD as u32) - (now % (score::SLOT_PERIOD as u32))) as u64;
            tokio::time::sleep(Duration::from_secs(duration)).await;

            // get the current epoch
            log::current(runtime).await;
            let timeslot = block::timeslot();
            let epoch = timeslot / score::EPOCH_LENGTH;
            let prev = timeslot.saturating_sub(1);
            if handshake.head.slot < prev {
                // select the best chain before authoring
                if let Err(e) = runtime.select_best_chain(prev).await {
                    tracing::error!("Failed to select best chain: {:?}", e);
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

            // send ticket
            if let Some(ticket) = ticket {
                tokio::spawn({
                    let runtime = runtime.clone();
                    async move { runtime.ticket(epoch, ticket).await }
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
