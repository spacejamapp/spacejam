//! Network implementation of Spacejam.

use std::{pin::Pin, rc::Rc};

pub use config::Config;
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
use tokio::sync::RwLock;
use tokio_stream::Stream;

pub mod config;
mod event;
mod peer;

const BLOCK_NAME: &str = "/notif/block/1";
const BLOCK_SYNC_NAME: &str = "/sync/block/1";
const STATE_SYNC_NAME: &str = "/sync/state/1";

/// Network implementation of Spacejam.
pub struct Network {
    /// P2P instance.
    pub p2p: Rc<RwLock<Litep2p>>,

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
}

impl Network {
    /// Start the network.
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let (block, block_handle) = config.block(BLOCK_NAME, &[]);
        let (block_sync, block_sync_handle) = config.block_sync(BLOCK_SYNC_NAME, &[]);
        let (state_sync, state_sync_handle) = config.state_sync(STATE_SYNC_NAME, &[]);
        let (ping, ping_handle) = ping::ConfigBuilder::new().with_max_failure(10).build();
        let (kad, kad_handle) = kademlia::ConfigBuilder::new().build();
        let (mdns, mdns_handle) = mdns::Config::new(config.mdns);

        // Create the network instance
        let p2p = Rc::new(RwLock::new(Litep2p::new(
            ConfigBuilder::new()
                .with_libp2p_ping(ping)
                .with_libp2p_kademlia(kad)
                .with_mdns(mdns)
                .with_quic(config.quic())
                .with_notification_protocol(block)
                .with_request_response_protocol(block_sync)
                .with_request_response_protocol(state_sync)
                .build(),
        )?));

        let _ = tracing::span!(tracing::Level::INFO, "bootstrap").enter();
        for address in config.bootstrap.iter() {
            tracing::event!(tracing::Level::INFO, "dialing {address:?}");
            if let Err(e) = p2p.write().await.dial_address(address.clone()).await {
                tracing::warn!("failed to dial {address:?}: {e:?}");
            }
        }

        Ok(Self {
            p2p,
            block: block_handle,
            ping: Box::pin(ping_handle),
            kad: kad_handle,
            mdns: Box::pin(mdns_handle),
            sync: block_sync_handle,
            state: state_sync_handle,
        })
    }

    /// Start the network.
    ///
    /// TODO: dial registered addresses.
    pub async fn start(&mut self) {
        tracing::info!(
            "listen addresses: {:?}",
            self.p2p.read().await.listen_addresses().collect::<Vec<_>>()
        );

        tokio::select! {
            _ = Self::spawn_litep2p(Rc::clone(&self.p2p)) => {}
            _ = self.spawn_events() => {}
        }
    }
}
