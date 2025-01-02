//! Network implementation of Spacejam.

pub use config::Config;
use litep2p::{
    config::ConfigBuilder,
    protocol::{notification::NotificationHandle, request_response::RequestResponseHandle},
    Litep2p,
};
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;

pub mod config;

const BLOCK_ANNOUNCE_NAME: &str = "/notif/block-announce/1";
const BLOCK_SYNC_NAME: &str = "/sync/block/1";
const STATE_SYNC_NAME: &str = "/sync/state/1";

/// Network implementation of Spacejam.
pub struct Network {
    /// P2P instance.
    pub p2p: Litep2p,

    /// Task handle.
    pub task: JoinHandle<()>,
}

impl Network {
    /// Start the network.
    pub async fn new(config: Config) -> anyhow::Result<Network> {
        let (block_announce, mut block_announce_handle) =
            config.block_announce(BLOCK_ANNOUNCE_NAME, &[]);
        let (block_sync, mut block_sync_handle) = config.block_sync(BLOCK_SYNC_NAME, &[]);
        let (state_sync, mut state_sync_handle) = config.state_sync(STATE_SYNC_NAME, &[]);

        // create a p2p instance
        let pconfig = ConfigBuilder::new()
            .with_quic(config.quic())
            .with_notification_protocol(block_announce)
            .with_request_response_protocol(block_sync)
            .with_request_response_protocol(state_sync)
            .build();

        Ok(Network {
            p2p: Litep2p::new(pconfig)?,
            task: tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = block_announce_handle.next() => {}
                        _ = block_sync_handle.next() => {}
                        _ = state_sync_handle.next() => {}
                    }
                }
            }),
        })
    }

    /// Start the network.
    pub async fn start(mut self) -> anyhow::Result<()> {
        while let Some(event) = self.p2p.next_event().await {
            println!("event: {:?}", event);
        }

        Ok(())
    }
}

impl Drop for Network {
    fn drop(&mut self) {
        self.task.abort();
    }
}
