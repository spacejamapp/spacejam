//! Client implementation for SpaceJam with QUIC.

use crate::config::Config;
use anyhow::Result;
use quinn::{crypto::rustls::QuicClientConfig, ClientConfig, Endpoint, TransportConfig};
use rustls::RootCertStore;
use std::{net::SocketAddr, sync::Arc};

pub struct Client {
    endpoint: Endpoint,
}

impl Client {
    /// Create a new QUIC client with the given configuration
    pub fn new(config: &Config) -> Result<Self> {
        let client_config = Self::configure(config)?;
        let mut endpoint = Endpoint::client(config.client.addr.into())?;
        endpoint.set_default_client_config(client_config);

        Ok(Self { endpoint })
    }

    /// Configure client with TLS settings
    fn configure(config: &Config) -> Result<ClientConfig> {
        let (cert, _) = config.client.der.load()?;
        let mut root = RootCertStore::empty();
        for cert in cert {
            root.add(cert)?;
        }

        let crypto = rustls::ClientConfig::builder()
            .with_root_certificates(Arc::new(root))
            .with_no_client_auth();

        let conf = QuicClientConfig::try_from(Arc::new(crypto))?;
        let mut client_config = ClientConfig::new(Arc::new(conf));
        let mut transport = TransportConfig::default();
        transport
            .max_concurrent_uni_streams(0_u8.into())
            .max_concurrent_bidi_streams(1_u8.into());
        client_config.transport_config(Arc::new(transport));

        Ok(client_config)
    }

    /// Connect to a QUIC server and send a request
    pub async fn request(&self, addr: SocketAddr, request: Vec<u8>) -> Result<Vec<u8>> {
        tracing::trace!("connecting to {}", addr);
        let connection = self.endpoint.connect(addr, "spacejam")?.await?;

        // Open a single bidirectional stream as per JAMNP-S
        tracing::trace!("connected to {}", connection.remote_address());
        let (mut send, mut recv) = connection.open_bi().await?;

        // Send the request
        send.write_all(&request).await?;
        send.finish()?;

        // Read the response
        let response = recv.read_to_end(64 * 1024).await?;
        Ok(response)
    }
}
