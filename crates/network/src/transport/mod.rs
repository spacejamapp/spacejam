//! Transport implementation for Spacejam.

pub use builder::Builder;
use crypto::ed25519;
use verifier::Verifier;

mod builder;
mod verifier;

/// Transport implementation for Spacejam.
pub struct Transport;

impl Transport {
    /// Create a new builder.
    pub fn builder(keypair: ed25519::KeyPair) -> builder::Builder {
        builder::Builder::new(keypair)
    }
}
