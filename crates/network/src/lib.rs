//! Network implementation of Spacejam.

use litep2p::{
    config::ConfigBuilder,
    crypto::ed25519::Keypair,
    protocol::{
        libp2p::{
            kademlia::{self, KademliaHandle},
            ping::{self, PingEvent},
        },
        mdns::{self, MdnsEvent},
        notification::NotificationHandle,
        request_response::RequestResponseHandle,
    },
    Litep2p,
};
use metrics::Metrics;
use peer::PeerManager;
use std::{pin::Pin, sync::Arc, time::Duration};
use tokio_stream::{Stream, StreamExt};
pub use {config::Config, context::Context, litep2p::crypto::ed25519};

pub mod config;
mod context;
mod event;
mod peer;

const BLOCK_NAME: &str = "/notif/block/1";
const BLOCK_SYNC_NAME: &str = "/sync/block/1";
const STATE_SYNC_NAME: &str = "/sync/state/1";

/// Network implementation of Spacejam.
pub struct Network {
    /// P2P instance.
    pub p2p: Litep2p,

    /// Block handle.
    block: NotificationHandle,

    /// Ping handle.
    ping: Pin<Box<dyn Stream<Item = PingEvent>>>,

    /// Kademlia handle.
    kad: KademliaHandle,

    /// mDNS handle.
    mdns: Pin<Box<dyn Stream<Item = MdnsEvent>>>,

    /// Sync handle.
    sync: RequestResponseHandle,

    /// State handle.
    state: RequestResponseHandle,

    /// Peer manager.
    peer: PeerManager,

    /// Metrics.
    pub metrics: Arc<Metrics>,
}

impl Network {
    /// Create a new network instance.
    pub async fn new(config: Config, keypair: Option<Keypair>) -> anyhow::Result<Self> {
        let (block, block_handle) = config.block(BLOCK_NAME, &[]);
        let (block_sync, block_sync_handle) = config.block_sync(BLOCK_SYNC_NAME, &[]);
        let (state_sync, state_sync_handle) = config.state_sync(STATE_SYNC_NAME, &[]);
        let (ping, ping_handle) = ping::ConfigBuilder::new().with_max_failure(10).build();
        let (kad, kad_handle) = kademlia::ConfigBuilder::new().build();
        let (mdns, mdns_handle) = mdns::Config::new(Duration::from_secs(config.mdns));

        // Create the network instance
        let mut p2p = {
            let mut builder = ConfigBuilder::new();
            if let Some(kp) = keypair {
                builder = builder.with_keypair(kp);
            }

            Litep2p::new(
                builder
                    .with_libp2p_ping(ping)
                    .with_libp2p_kademlia(kad)
                    .with_mdns(mdns)
                    .with_quic(config.quic())
                    .with_notification_protocol(block)
                    .with_request_response_protocol(block_sync)
                    .with_request_response_protocol(state_sync)
                    .build(),
            )?
        };

        // Logging addresses
        //
        // TODO: quic port dispatch (default value && port occupied)
        if config.quic.addresses.len() == 1 {
            tracing::info!("listen on: {}", config.quic.addresses[0]);
        } else if !config.quic.addresses.is_empty() {
            tracing::info!("listen on: {:?}", config.quic.addresses);
        }

        // Dial the bootstrap addresses
        for address in config.bootstrap.iter() {
            tracing::event!(tracing::Level::INFO, "dialing {address:?}");
            if let Err(e) = p2p.dial_address(address.clone()).await {
                tracing::warn!("failed to dial {address:?}: {e:?}");
            }
        }

        Ok(Self {
            block: block_handle,
            ping: Box::pin(ping_handle),
            kad: kad_handle,
            mdns: Box::pin(mdns_handle),
            sync: block_sync_handle,
            state: state_sync_handle,
            peer: PeerManager::default(),
            metrics: Arc::new(Metrics::new(&p2p.local_peer_id().to_string())),
            p2p,
        })
    }

    /// Spawn the network.
    pub async fn spawn(&mut self, context: &impl Context) {
        loop {
            tokio::select! {
                Some(event) = self.block.next() => self.block(event, context),
                Some(event) = self.sync.next() => self.sync(event),
                Some(event) = self.state.next() => self.state(event),
                Some(event) = self.ping.next() => self.ping(event).await,
                Some(event) = self.kad.next() => self.kad(event).await,
                Some(event) = self.mdns.next() => self.mdns(event).await,
                Some(event) = self.p2p.next_event() => self.litep2p(event).await,
            }
        }
    }
}
