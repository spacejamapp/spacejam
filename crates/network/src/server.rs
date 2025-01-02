//! Server implementation for SpaceJam with QUIC.

use crate::config::Config;
use anyhow::Result;
use quinn::{crypto::rustls::QuicServerConfig, Endpoint, ServerConfig};
use std::sync::Arc;

/// The server implementation for SpaceJam with QUIC
pub struct Server {
    endpoint: Endpoint,
}

impl Server {
    /// Create a new QUIC server with the given configuration
    pub fn new(config: &Config) -> Result<Self> {
        let (certs, key) = config.server.der.load()?;
        let server_crypto = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)?;

        let mut server_config = ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(
            Arc::new(server_crypto),
        )?));

        if let Some(transport) = Arc::get_mut(&mut server_config.transport) {
            transport
                .max_concurrent_uni_streams(0_u8.into())
                .max_concurrent_bidi_streams(1_u8.into());
        }

        Ok(Self {
            endpoint: Endpoint::server(server_config, config.server.addr.into())?,
        })
    }

    /// Start listening for incoming connections
    pub async fn run(&self) -> Result<()> {
        tracing::trace!("listening on {}", self.endpoint.local_addr()?);

        loop {
            let connection = self.endpoint.accept().await;
            let Some(connection) = connection else {
                tracing::warn!("server is shutting down");
                return Ok(());
            };

            let connection = connection.await?;
            tracing::trace!("connection accepted from {}", connection.remote_address());

            // Accept a single bidirectional stream as per JAMNP-S
            if let Ok(stream) = connection.accept_bi().await {
                tokio::spawn(async move {
                    if let Err(e) = handle_stream(stream).await {
                        tracing::warn!("error handling stream: {}", e);
                    }
                });
            }
        }
    }
}

async fn handle_stream((mut send, mut recv): (quinn::SendStream, quinn::RecvStream)) -> Result<()> {
    // Read the request
    let _request = recv.read_to_end(64 * 1024).await?;

    // Process the request and send response
    // TODO: Implement actual request handling according to JAMNP-S
    send.write_all(b"OK").await?;
    send.finish()?;
    Ok(())
}
