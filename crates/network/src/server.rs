//! Server implementation for SpaceJam with QUIC.

use anyhow::Result;
use quinn::{Endpoint, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::{net::SocketAddr, sync::Arc};

pub struct Server {
    endpoint: Endpoint,
}

impl Server {
    /// Create a new QUIC server with the given configuration
    pub async fn new(
        addr: SocketAddr,
        cert: impl Into<Vec<CertificateDer<'static>>>,
        key: PrivateKeyDer<'static>,
    ) -> Result<Self> {
        let server_config = Self::configure_server(cert, key)?;
        let endpoint = Endpoint::server(server_config, addr)?;

        Ok(Self { endpoint })
    }

    /// Configure server with TLS certificates
    fn configure_server(
        cert: impl Into<Vec<CertificateDer<'static>>>,
        key: PrivateKeyDer<'static>,
    ) -> Result<ServerConfig> {
        let mut server_config = ServerConfig::with_single_cert(cert.into(), key)?;
        Arc::get_mut(&mut server_config.transport)
            .unwrap()
            .max_concurrent_uni_streams(0_u8.into())
            .max_concurrent_bidi_streams(1_u8.into());

        Ok(server_config)
    }

    /// Start listening for incoming connections
    pub async fn run(&self) -> Result<()> {
        tracing::info!("listening on {}", self.endpoint.local_addr()?);

        loop {
            let connection = self.endpoint.accept().await;
            let Some(connection) = connection else {
                tracing::warn!("server is shutting down");
                return Ok(());
            };

            let connection = connection.await?;
            tracing::info!("connection accepted from {}", connection.remote_address());

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
