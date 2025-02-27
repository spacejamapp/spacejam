//! Node for SpaceJam

use network::{Event, Network};
use score::runtime::{Storage, Validator};
use std::net::SocketAddr;
use tokio::sync::mpsc;
pub use {builder::Builder, context::Context, genesis::Genesis};

mod author;
mod builder;
mod context;
mod genesis;
pub mod metrics;

/// Start the node
///
/// TODO: make metrics service out of this function?
pub async fn start<S: Storage + Send + Sync + 'static, V: Validator + Send + Sync + 'static>(
    network: Network<Context<S, V>>,
    rx: mpsc::UnboundedReceiver<Event>,
    metrics: SocketAddr,
) -> anyhow::Result<()> {
    let context = network.context.clone();

    tokio::select! {
        _ = metrics::serve(metrics, context.metrics.clone()) => {}
        _ = author::run(&context) => {}
        _ = network.spawn(rx) => {}
        _ = tokio::signal::ctrl_c() => {}
    }

    Ok(())
}
