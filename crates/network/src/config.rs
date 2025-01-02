//! Configuration for the network.

use litep2p::{
    protocol::{self, notification::NotificationHandle, request_response::RequestResponseHandle},
    transport::quic,
    types::multiaddr::Multiaddr,
    ProtocolName,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    block_announce: NotifiConfig,
    block_sync: SyncConfig,
    quic: QuicConfig,
    state_sync: SyncConfig,
}

impl Config {
    /// Get the QUIC configuration.
    pub fn quic(&self) -> quic::config::Config {
        self.quic.clone().into()
    }

    /// Get the notification configuration.
    pub fn block_announce(
        &self,
        name: &'static str,
        fallback_names: &[&'static str],
    ) -> (protocol::notification::Config, NotificationHandle) {
        protocol::notification::Config::new(
            name.into(),
            self.block_announce.max_notification_size,
            self.block_announce.handshake.clone(),
            fallback_names
                .into_iter()
                .map(|s| ProtocolName::Static(s))
                .collect(),
            self.block_announce.auto_accept,
            self.block_announce.sync_channel_size,
            self.block_announce.async_channel_size,
            self.block_announce.should_dial,
        )
    }

    /// Get the request-response configuration.
    pub fn block_sync(
        &self,
        name: &'static str,
        fallback_names: &[&'static str],
    ) -> (protocol::request_response::Config, RequestResponseHandle) {
        self.req_resp(&self.block_sync, name, fallback_names)
    }

    /// Get the request-response configuration.
    pub fn state_sync(
        &self,
        name: &'static str,
        fallback_names: &[&'static str],
    ) -> (protocol::request_response::Config, RequestResponseHandle) {
        self.req_resp(&self.state_sync, name, fallback_names)
    }

    /// Get the request-response configuration.
    pub fn req_resp(
        &self,
        config: &SyncConfig,
        name: &'static str,
        fallback_names: &[&'static str],
    ) -> (protocol::request_response::Config, RequestResponseHandle) {
        protocol::request_response::Config::new(
            name.into(),
            fallback_names
                .into_iter()
                .map(|s| ProtocolName::Static(s))
                .collect(),
            config.max_message_size,
            config.timeout,
            config.max_concurrent_inbound_request,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuicConfig {
    addresses: Vec<Multiaddr>,
    connection: Duration,
    substream: Duration,
}

impl From<QuicConfig> for quic::config::Config {
    fn from(config: QuicConfig) -> Self {
        Self {
            listen_addresses: config.addresses,
            connection_open_timeout: config.connection,
            substream_open_timeout: config.substream,
        }
    }
}

/// Configuration for the notification protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifiConfig {
    /// The maximum size of a notification.
    pub max_notification_size: usize,

    /// The handshake message.
    pub handshake: Vec<u8>,

    /// Whether to automatically accept notifications.
    pub auto_accept: bool,

    /// The size of the sync channel.
    pub sync_channel_size: usize,

    /// The size of the async channel.
    pub async_channel_size: usize,

    /// Whether to dial to the peer.
    pub should_dial: bool,
}

impl Default for NotifiConfig {
    fn default() -> Self {
        Self {
            max_notification_size: 1024 * 1024,
            handshake: vec![42],
            auto_accept: true,
            sync_channel_size: 100,
            async_channel_size: 100,
            should_dial: true,
        }
    }
}

/// Configuration for the request-response protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// The maximum size of a message.
    pub max_message_size: usize,

    /// The timeout for a request.
    pub timeout: Duration,

    /// The maximum number of concurrent inbound requests.
    pub max_concurrent_inbound_request: Option<usize>,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            max_message_size: 1024 * 1024,
            timeout: Duration::from_secs(60),
            max_concurrent_inbound_request: None,
        }
    }
}
