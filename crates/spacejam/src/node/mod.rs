//! Node for SpaceJam

use network::{Event, Network};
use std::net::SocketAddr;
use tokio::sync::mpsc;
pub use {builder::Builder, genesis::Genesis};

mod author;
mod builder;
mod genesis;
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
        _ = author::run(&runtime) => {}
        _ = network.spawn(rx) => {}
        _ = tokio::signal::ctrl_c() => {}
    }

    Ok(())
}
