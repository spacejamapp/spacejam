//! Network implementation of Spacejam.

use std::sync::Arc;
use tokio::sync::mpsc;
pub use {
    config::Config,
    context::Context,
    event::Event,
    peer::{Address, Manager},
    transport::{Builder as TransportBuilder, Transport},
};

mod config;
mod context;
pub mod event;
pub mod peer;
mod stream;
mod transport;

/// The network protocol name of Spacejam.
pub const PROTOCOL: &str = "jamnp-s";

/// Network implementation of Spacejam.
pub struct Network {
    /// Event receiver
    prx: mpsc::UnboundedReceiver<event::peer::Event>,

    /// Action receiver
    arx: mpsc::UnboundedReceiver<event::action::Event>,
}

impl Network {
    /// Create a new network.
    ///
    /// If the provided keypair is not provided, the node will not
    /// be a validator.
    pub async fn new<C: Context + Send + Sync + 'static>(
        config: Config,
        context: Arc<C>,
        arx: mpsc::UnboundedReceiver<event::action::Event>,
        prx: mpsc::UnboundedReceiver<event::peer::Event>,
    ) -> anyhow::Result<Self> {
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
        }

        transport.spawn().await?;
        Ok(Self { arx, prx })
    }

    /// Spawn the network
    pub async fn spawn<C: Context + Send + Sync + 'static>(self, context: Arc<C>) {
        // Spawn the event handling loop
        let mut arx = self.arx;
        let mut prx = self.prx;

        loop {
            let ctx = context.clone();
            let e = tokio::select! {
                Some(act) = arx.recv() => Event::Action(act),
                Some(ev) = prx.recv() => Event::Peer(ev),
                else => {
                    tracing::error!("all channels closed, terminating event loop");
                    break;
                }
            };

            e.handle_unchecked(ctx).await;
        }
    }
}

/// Pick a random port.
pub fn pick() -> std::io::Result<u16> {
    use std::net::UdpSocket;

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    let addr = socket.local_addr()?;
    Ok(addr.port())
}
