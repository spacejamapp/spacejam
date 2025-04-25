//! Node for SpaceJam

use crate::{offchain::Offchain, utils::log};
use network::Network;
use runtime::storage::Storage;
use score::{block, safrole::ValidatorIter};
use std::{net::SocketAddr, time::Duration};
pub use {builder::Builder, genesis::Genesis};

mod builder;
mod genesis;

#[allow(unused)]
/// The specification of the node
pub enum NodeSpec {
    /// Validating node
    Validating,

    /// Light node
    Light,

    /// Development node
    Dev,

    /// Testing node
    Testing,
}

/// Start the node
///
/// TODO: make metrics service out of this function?
pub async fn start<C: runtime::Config>(
    network: Network<C>,
    metrics: SocketAddr,
    rpc: SocketAddr,
) -> anyhow::Result<()> {
    let runtime = network.clone();
    let offchain = Offchain::new(network.runtime.clone());

    tokio::select! {
        _ = author(&runtime) => {}
        _ = offchain.start(rpc, network.metrics.clone(), metrics) => {}
        _ = network.spawn() => {}
        _ = tokio::signal::ctrl_c() => {}
    }

    Ok(())
}

/// Authoring service
#[tracing::instrument(skip_all, name = "author")]
async fn author<C: runtime::Config>(runtime: &Network<C>) {
    log::init(runtime).await;
    let mut author = runtime.author();
    if let Err(e) = author.on_new_epoch().await {
        tracing::error!("Failed to initialize authoring: {e:?}");
        return;
    }

    // sleep up to 3 seconds to make sure the network is ready
    tokio::time::sleep(Duration::from_secs(3)).await;

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
                (score::SLOT_PERIOD * score::EPOCH_LENGTH
                    - now % (score::SLOT_PERIOD * score::EPOCH_LENGTH)) as u64,
            ))
            .await;
            continue;
        }

        // sleep until the next slot
        let duration = (score::SLOT_PERIOD - (now % score::SLOT_PERIOD)) as u64;
        tokio::time::sleep(Duration::from_secs(duration)).await;

        // get the current epoch
        log::current(runtime).await;
        let timeslot = block::timeslot();
        let epoch = timeslot / score::EPOCH_LENGTH;

        // select the best chain before authoring
        if let Err(e) = runtime.select_best_chain(timeslot).await {
            tracing::error!("Failed to select best chain: {:?}", e);
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
            if let Err(e) = runtime.ticket(epoch, ticket).await {
                tracing::error!("Failed to send ticket: {:?}", e);
            }
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
