//! Transport implementation for Spacejam.

use crypto::ed25519;
pub use {builder::Builder, verifier::Verifier};

mod builder;
mod verifier;

/// Create a new builder.
pub fn builder(keypair: ed25519::KeyPair) -> builder::Builder {
    builder::Builder::new(keypair)
}

/// Pick a random port.
pub fn pick() -> std::io::Result<u16> {
    use std::net::UdpSocket;

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    let addr = socket.local_addr()?;
    Ok(addr.port())
}
