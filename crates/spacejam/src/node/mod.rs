//! Node for SpaceJam

use crate::offchain::Offchain;
use network::{Event, Network};
use score::block;
use std::{net::SocketAddr, time::Duration};
use tokio::sync::mpsc;

pub use {builder::Builder, genesis::Genesis};

mod builder;
mod genesis;
mod log;

/// Start the node
///
/// TODO: make metrics service out of this function?
pub async fn start<C: runtime::Config>(
    network: Network<C>,
    rx: mpsc::UnboundedReceiver<Event>,
    metrics: SocketAddr,
    rpc: SocketAddr,
) -> anyhow::Result<()> {
    let runtime = network.clone();
    let offchain = Offchain::new(network.runtime.clone());

    tokio::select! {
        _ = author(&runtime) => {}
        _ = offchain.start(rpc, network.metrics.clone(), metrics) => {}
        _ = network.spawn(rx) => {}
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

    // sleep for 10 seconds to make sure the network is ready
    tokio::time::sleep(Duration::from_secs(10)).await;

    loop {
        let now = block::now().expect("failed to get current time");
        let duration = (score::SLOT_PERIOD - (now % score::SLOT_PERIOD)) as u64;
        tokio::time::sleep(Duration::from_secs(duration)).await;

        // get the current epoch
        log::current(runtime).await;
        let timeslot = block::timeslot().expect("failed to get current timeslot");
        let epoch = timeslot / score::EPOCH_LENGTH;

        // select the best chain before authoring
        if let Err(e) = network::event::sync::select_best_chain(runtime.clone(), timeslot).await {
            tracing::error!("Failed to select best chain: {:?}", e);
        }

        // author block and maybe generate ticket
        let (header, ticket) = match author.next().await {
            Ok((header, ticket)) => (header, ticket),
            Err(e) => {
                tracing::error!("Authoring error: {:?}", e);
                continue;
            }
        };

        // send ticket
        if let Some(ticket) = ticket {
            if let Err(e) = runtime.send(Event::DistributeTicket {
                epoch,
                ticket: Box::new(ticket),
            }) {
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

            if let Err(e) =
                network::event::broadcast::announce(runtime.clone(), Box::new(header.clone())).await
            {
                tracing::error!("Failed to announce block: {:?}", e);
            }
        }
    }
}
