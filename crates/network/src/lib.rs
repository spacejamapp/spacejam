//! Network implementation of Spacejam.

use std::sync::Arc;
pub use {
    config::Config,
    context::Context,
    event::Event,
    handle::Handle,
    peer::{Address, Manager},
    transport::{Builder as TransportBuilder, Transport},
};

mod config;
mod context;
pub mod event;
mod handle;
pub mod peer;
mod stream;
mod transport;

/// The network protocol name of Spacejam.
pub const PROTOCOL: &str = "jamnp-s";

/// Initialize the network
pub async fn init<C: Context + Send + Sync + 'static>(
    config: Config,
    context: Arc<C>,
) -> anyhow::Result<()> {
    let transport = Transport::builder(context.keypair().unwrap_or_default())
        .address(config.address)
        .genesis(config.genesis)
        .build(context.manager().read().await.ptx.clone())?;

    // Spawn a task to handle bootstrap dialing
    let bootstrap = config.bootstrap;
    if !bootstrap.is_empty() {
        for peer in bootstrap {
            tracing::debug!("dialing bootstrap peer: {peer}");
            if let Err(e) = transport.dial(peer).await {
                tracing::warn!("failed to dial bootstrap peer: {e}");
            }
        }
    } else {
        tracing::debug!("no bootstrap peers, skipping");
    }

    transport.spawn().await?;
    Ok(())
}

/// Pick a random port.
pub fn pick() -> std::io::Result<u16> {
    use std::net::UdpSocket;

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    let addr = socket.local_addr()?;
    Ok(addr.port())
}
