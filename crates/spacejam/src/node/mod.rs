//! Node for SpaceJam

use network::Handle;
use score::runtime::{Storage, Validator};
use std::net::SocketAddr;
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
    handle: Handle<Context<S, V>>,
    metrics: SocketAddr,
) -> anyhow::Result<()> {
    let context = handle.context.clone();

    tokio::select! {
        _ = metrics::serve(metrics, context.metrics.clone()) => {}
        _ = author::run(&context) => {}
        _ = handle.spawn() => {}
        _ = tokio::signal::ctrl_c() => {}
    }

    Ok(())
}
