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
    notification: NotificationConfig,
    quic: QuicConfig,
    #[serde(rename = "req-resp")]
    req_resp: RequestResponseConfig,
}

impl Config {
    /// Get the QUIC configuration.
    pub fn quic(&self) -> quic::config::Config {
        self.quic.clone().into()
    }

    /// Get the request-response configuration.
    pub fn req_resp(
        &self,
        name: &'static str,
        fallback_names: &[&'static str],
    ) -> (protocol::request_response::Config, RequestResponseHandle) {
        protocol::request_response::Config::new(
            name.into(),
            fallback_names
                .into_iter()
                .map(|s| ProtocolName::Static(s))
                .collect(),
            self.req_resp.max_message_size,
            self.req_resp.timeout,
            self.req_resp.max_concurrent_inbound_request,
        )
    }

    /// Get the notification configuration.
    pub fn notification(
        &self,
        name: &'static str,
        fallback_names: &[&'static str],
    ) -> (protocol::notification::Config, NotificationHandle) {
        protocol::notification::Config::new(
            name.into(),
            self.notification.max_notification_size,
            self.notification.handshake.clone(),
            fallback_names
                .into_iter()
                .map(|s| ProtocolName::Static(s))
                .collect(),
            self.notification.auto_accept,
            self.notification.sync_channel_size,
            self.notification.async_channel_size,
            self.notification.should_dial,
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
pub struct NotificationConfig {
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

/// Configuration for the request-response protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestResponseConfig {
    /// The maximum size of a message.
    pub max_message_size: usize,

    /// The timeout for a request.
    pub timeout: Duration,

    /// The maximum number of concurrent inbound requests.
    pub max_concurrent_inbound_request: Option<usize>,
}
