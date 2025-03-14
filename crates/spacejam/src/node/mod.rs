//! Node for SpaceJam

use network::{Event, Network};
use score::block;
use std::{net::SocketAddr, time::Duration};
use tokio::sync::mpsc;
pub use {builder::Builder, genesis::Genesis};

mod builder;
mod genesis;
mod log;
pub mod metrics;

/// Start the node
///
/// TODO: make metrics service out of this function?
pub async fn start<C: score::runtime::Config>(
    network: Network<C>,
    rx: mpsc::UnboundedReceiver<Event>,
    metrics: SocketAddr,
) -> anyhow::Result<()> {
    let runtime = network.clone();

    tokio::select! {
        _ = metrics::serve(metrics, network.metrics.clone()) => {}
        _ = author(&runtime) => {}
        _ = network.spawn(rx) => {}
        _ = tokio::signal::ctrl_c() => {}
    }

    Ok(())
}

/// Authoring service
async fn author<C: score::runtime::Config>(runtime: &Network<C>) {
    log::init(runtime).await;
    let mut author = runtime.author();
    loop {
        log::current(runtime, &author.validators).await;
        let Ok(now) = block::now() else {
            tracing::error!("Failed to get current time");
            tokio::time::sleep(Duration::from_secs(score::SLOT_PERIOD as u64)).await;
            continue;
        };

        // get the current epoch
        let epoch = now / score::SLOT_PERIOD / score::EPOCH_LENGTH;

        // author block and maybe generate ticket
        let duration = (score::SLOT_PERIOD - (now % score::SLOT_PERIOD)) as u64;
        let next = author.next().await;
        if let Err(e) = next {
            tracing::error!("Authoring error: {:?}", e);
            tokio::time::sleep(Duration::from_secs(duration as u64)).await;
            continue;
        }

        // get the authoring result
        let (header, ticket) = next.expect("checked before");

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
            if let Err(e) = runtime.send(Event::AnnounceBlock(Box::new(header))) {
                tracing::error!("Failed to announce block: {:?}", e);
            }
        }

        // sleep for the next slot
        tokio::time::sleep(Duration::from_secs(duration as u64)).await;
    }
}
