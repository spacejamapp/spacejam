//! Network implementation of Spacejam.

use litep2p::{
    config::ConfigBuilder,
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
use std::{pin::Pin, time::Duration};
use tokio_stream::{Stream, StreamExt};
pub use {config::Config, context::Context};

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

    /// Context.
    pub context: Box<dyn Context>,

    /// Metrics.
    pub metrics: Metrics,
}

impl Network {
    /// Create a new network instance.
    pub async fn new(config: Config, context: Box<dyn Context>) -> anyhow::Result<Self> {
        let (block, block_handle) = config.block(BLOCK_NAME, &[]);
        let (block_sync, block_sync_handle) = config.block_sync(BLOCK_SYNC_NAME, &[]);
        let (state_sync, state_sync_handle) = config.state_sync(STATE_SYNC_NAME, &[]);
        let (ping, ping_handle) = ping::ConfigBuilder::new().with_max_failure(10).build();
        let (kad, kad_handle) = kademlia::ConfigBuilder::new().build();
        let (mdns, mdns_handle) = mdns::Config::new(Duration::from_secs(config.mdns));

        // Create the network instance
        let p2p = Litep2p::new(
            ConfigBuilder::new()
                .with_libp2p_ping(ping)
                .with_libp2p_kademlia(kad)
                .with_mdns(mdns)
                .with_quic(config.quic())
                .with_notification_protocol(block)
                .with_request_response_protocol(block_sync)
                .with_request_response_protocol(state_sync)
                .build(),
        )?;

        // Create the network instance
        let mut this = Self {
            block: block_handle,
            ping: Box::pin(ping_handle),
            kad: kad_handle,
            mdns: Box::pin(mdns_handle),
            sync: block_sync_handle,
            state: state_sync_handle,
            peer: PeerManager::default(),
            metrics: Metrics::new(&p2p.local_peer_id().to_string()),
            context,
            p2p,
        };

        // Bootstrap the network
        this.bootstrap(&config).await;
        Ok(this)
    }

    /// Spawn the network.
    pub async fn spawn(&mut self) {
        let listen_addresses = self.p2p.listen_addresses().collect::<Vec<_>>();
        tracing::info!("listen addresses: {listen_addresses:?}");

        loop {
            tokio::select! {
                Some(event) = self.block.next() => self.block(event),
                Some(event) = self.sync.next() => self.sync(event),
                Some(event) = self.state.next() => self.state(event),
                Some(event) = self.ping.next() => self.ping(event).await,
                Some(event) = self.kad.next() => self.kad(event).await,
                Some(event) = self.mdns.next() => self.mdns(event).await,
                Some(event) = self.p2p.next_event() => self.litep2p(event).await,
            }
        }
    }

    /// Bootstrap the network.
    async fn bootstrap(&mut self, config: &Config) {
        for address in config.bootstrap.iter() {
            tracing::event!(tracing::Level::INFO, "dialing {address:?}");
            if let Err(e) = self.p2p.dial_address(address.clone()).await {
                tracing::warn!("failed to dial {address:?}: {e:?}");
            }
        }
    }
}
