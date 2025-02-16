//! Transport implementation for Spacejam.

use crate::Event;
use crypto::ed25519;
use quinn::{crypto::rustls::HandshakeData, Endpoint};
pub use {builder::Builder, verifier::Verifier};

mod builder;
mod verifier;

/// Transport implementation for Spacejam.
pub struct Transport {
    /// QUIC endpoint.
    endpoint: Endpoint,
}

impl Transport {
    /// Create a new builder.
    pub fn builder(keypair: ed25519::KeyPair) -> builder::Builder {
        builder::Builder::new(keypair)
    }

    /// Accept a new connection.
    ///
    /// TODO: remove the unwraps
    pub async fn accept(&mut self) -> anyhow::Result<Event> {
        let conn = self.endpoint.accept().await.unwrap().await.unwrap();
        let data: Box<HandshakeData> = conn.handshake_data().unwrap().downcast().unwrap();
        let protocol = data.protocol.unwrap();

        if protocol != *b"jamnp-s/V/H" && protocol != *b"jamnp-s/V/H/builder" {
            return Err(anyhow::anyhow!("invalid protocol"));
        }

        let Some(server_name) = data.server_name else {
            return Err(anyhow::anyhow!("invalid server name"));
        };

        let mut peer = [0; 32];
        peer.copy_from_slice(
            &base32::decode(
                base32::Alphabet::Rfc4648Lower { padding: false },
                &server_name[1..],
            )
            .unwrap(),
        );

        // TODO: add this connection to pool.

        Ok(Event::ConnectionEstablished { peer })
    }
}
