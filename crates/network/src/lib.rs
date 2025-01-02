//! Network implementation of Spacejam.

use std::{pin::Pin, rc::Rc};

pub use config::Config;
use litep2p::{
    config::ConfigBuilder,
    protocol::{
        libp2p::ping::{self, PingEvent},
        notification::NotificationHandle,
        request_response::RequestResponseHandle,
    },
    Litep2p,
};
use tokio::sync::RwLock;
use tokio_stream::Stream;

pub mod config;
mod event;

const BLOCK_NAME: &str = "/notif/block/1";
const BLOCK_SYNC_NAME: &str = "/sync/block/1";
const STATE_SYNC_NAME: &str = "/sync/state/1";

/// Network implementation of Spacejam.
pub struct Network {
    /// P2P instance.
    pub p2p: Rc<RwLock<Litep2p>>,

    /// Block handle.
    block: NotificationHandle,

    /// Sync handle.
    sync: RequestResponseHandle,

    /// State handle.
    state: RequestResponseHandle,

    /// Ping handle.
    ping: Pin<Box<dyn Stream<Item = PingEvent>>>,
}

impl Network {
    /// Start the network.
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let (block, block_handle) = config.block(BLOCK_NAME, &[]);
        let (block_sync, block_sync_handle) = config.block_sync(BLOCK_SYNC_NAME, &[]);
        let (state_sync, state_sync_handle) = config.state_sync(STATE_SYNC_NAME, &[]);
        let (ping, ping_handle) = ping::ConfigBuilder::new().with_max_failure(10).build();

        // Create the network instance
        let p2p = Rc::new(RwLock::new(Litep2p::new(
            ConfigBuilder::new()
                .with_libp2p_ping(ping)
                .with_quic(config.quic())
                .with_notification_protocol(block)
                .with_request_response_protocol(block_sync)
                .with_request_response_protocol(state_sync)
                .build(),
        )?));

        Ok(Self {
            p2p,
            block: block_handle,
            sync: block_sync_handle,
            state: state_sync_handle,
            ping: Box::pin(ping_handle),
        })
    }

    /// Start the network.
    ///
    /// TODO: dial registered addresses.
    pub async fn start(&mut self) {
        tokio::select! {
            _ = Self::spawn_litep2p(Rc::clone(&self.p2p)) => {}
            _ = self.spawn_events() => {}
        }
    }
}
