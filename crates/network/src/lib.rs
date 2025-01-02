//! Network implementation of Spacejam.

use std::sync::Arc;

pub use config::Config;
use event::Event;
use litep2p::{config::ConfigBuilder, protocol::libp2p::ping, Litep2p};
use tokio::{
    sync::{
        mpsc::{self, Receiver},
        Mutex, RwLock,
    },
    task::JoinHandle,
};
use tokio_stream::StreamExt;

pub mod config;
mod event;

const BLOCK_NAME: &str = "/notif/block/1";
const BLOCK_SYNC_NAME: &str = "/sync/block/1";
const STATE_SYNC_NAME: &str = "/sync/state/1";

/// Network implementation of Spacejam.
pub struct Network {
    /// P2P instance.
    pub p2p: RwLock<Litep2p>,

    /// Event receiver.
    pub rx: Arc<Mutex<Receiver<Event>>>,

    /// Event handler.
    pub task: JoinHandle<()>,
}

impl Network {
    /// Start the network.
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let (block, mut block_handle) = config.block(BLOCK_NAME, &[]);
        let (block_sync, mut block_sync_handle) = config.block_sync(BLOCK_SYNC_NAME, &[]);
        let (state_sync, mut state_sync_handle) = config.state_sync(STATE_SYNC_NAME, &[]);
        let (ping, mut ping_handle) = ping::ConfigBuilder::new().with_max_failure(10).build();

        // Create the network instance
        let p2p = RwLock::new(Litep2p::new(
            ConfigBuilder::new()
                .with_libp2p_ping(ping)
                .with_quic(config.quic())
                .with_notification_protocol(block)
                .with_request_response_protocol(block_sync)
                .with_request_response_protocol(state_sync)
                .build(),
        )?);

        // Create the event channel
        let (tx, rx) = mpsc::channel(100);
        let rx = Arc::new(Mutex::new(rx));

        // Spawn the event handler
        let task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(event) = block_handle.next() => {
                        if let Err(e) = tx.send(Event::Block(event)).await {
                            tracing::error!("failed to send block announceevent: {e}");
                        }
                    }
                    Some(event) = block_sync_handle.next() => {
                        if let Err(e) = tx.send(Event::Sync(event)).await {
                            tracing::error!("failed to send block sync event: {e}");
                        }
                    }
                    Some(event) = state_sync_handle.next() => {
                        if let Err(e) = tx.send(Event::State(event)).await {
                            tracing::error!("failed to send state sync event: {e}");
                        }
                    }
                    Some(event) = ping_handle.next() => {
                        if let Err(e) = tx.send(Event::Ping(event)).await {
                            tracing::error!("failed to send ping event: {e}");
                        }
                    }
                }
            }
        });

        Ok(Self { p2p, rx, task })
    }

    /// Start the network.
    pub async fn start(&self) {
        tokio::select! {
            _ = self.spawn_events() => {}
            _ = self.spawn_litep2p() => {}
        }
    }
}
