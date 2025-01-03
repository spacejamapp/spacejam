//! Configuration for the network.

use litep2p::{
    protocol::{self, notification::NotificationHandle, request_response::RequestResponseHandle},
    transport::quic,
    types::multiaddr::{Multiaddr, Protocol},
    ProtocolName,
};
use serde::{Deserialize, Serialize};
use std::{net::Ipv4Addr, time::Duration};

/// Configuration for the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The block announce configuration.
    pub block: NotifiConfig,

    /// The block sync configuration.
    pub block_sync: SyncConfig,

    /// The QUIC configuration.
    pub quic: QuicConfig,

    /// The state sync configuration.
    pub state_sync: SyncConfig,

    /// The mDNS query interval.
    pub mdns: Duration,

    /// The bootstrap addresses.
    pub bootstrap: Vec<Multiaddr>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            block: NotifiConfig::default(),
            block_sync: SyncConfig::default(),
            quic: QuicConfig::default(),
            state_sync: SyncConfig::default(),
            mdns: Duration::from_secs(10),
            bootstrap: vec![],
        }
    }
}

impl Config {
    /// Get the QUIC configuration.
    pub fn quic(&self) -> quic::config::Config {
        self.quic.clone().into()
    }

    /// Get the notification configuration.
    pub fn block(
        &self,
        name: &'static str,
        fallback_names: &[&'static str],
    ) -> (protocol::notification::Config, NotificationHandle) {
        protocol::notification::Config::new(
            name.into(),
            self.block.max_notification_size,
            self.block.handshake.clone(),
            fallback_names
                .iter()
                .map(|s| ProtocolName::Static(s))
                .collect(),
            self.block.auto_accept,
            self.block.sync_channel_size,
            self.block.async_channel_size,
            self.block.should_dial,
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
                .iter()
                .map(|s| ProtocolName::Static(s))
                .collect(),
            config.max_message_size,
            config.timeout,
            config.max_concurrent_inbound_request,
        )
    }
}

/// Configuration for the QUIC transport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuicConfig {
    /// The addresses to listen on.
    pub addresses: Vec<Multiaddr>,

    /// The timeout for a connection.
    connection: Duration,

    /// The timeout for a substream.
    substream: Duration,
}

impl Default for QuicConfig {
    fn default() -> Self {
        let mut addr = Multiaddr::empty();
        addr.push(Protocol::Ip4(Ipv4Addr::LOCALHOST));
        addr.push(Protocol::Udp(0));
        addr.push(Protocol::QuicV1);

        Self {
            addresses: vec![addr],
            connection: Duration::from_secs(10),
            substream: Duration::from_secs(10),
        }
    }
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
