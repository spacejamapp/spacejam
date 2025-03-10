//! Peer related stuffs

use score::OpaqueHash;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
pub use {address::Address, conn::Connection};

mod address;
mod conn;

/// Peer ID, also known as the DNS name of the peer.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub struct PeerId(OpaqueHash);

impl PeerId {
    /// Create a new peer id.
    pub fn verify(id: &str) -> Result<Self, anyhow::Error> {
        let len = id.len();
        if len != 53 {
            anyhow::bail!("invalid peer id length, should be 53, got {len}");
        }

        if !id.starts_with("e") {
            anyhow::bail!(
                "invalid peer id prefix, should be 'e', got '{:?}'",
                id.chars().next()
            );
        }

        // Check if the peer id is valid base32
        let base32 = id.split_at(1).1;
        base32::decode(base32::Alphabet::Rfc4648Lower { padding: false }, base32)
            .ok_or_else(|| anyhow::anyhow!("peer id is not valid base32"))
            .and_then(|bytes| {
                OpaqueHash::try_from(bytes)
                    .map_err(|_| anyhow::anyhow!("failed to convert peer id bytes to [u8; 32]"))
                    .map(Into::into)
            })
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "e{}",
            base32::encode(base32::Alphabet::Rfc4648Lower { padding: false }, &self.0)
        )
    }
}

impl AsRef<OpaqueHash> for PeerId {
    fn as_ref(&self) -> &OpaqueHash {
        &self.0
    }
}

impl FromStr for PeerId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::verify(s)
    }
}

impl From<[u8; 32]> for PeerId {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

impl From<PeerId> for [u8; 32] {
    fn from(value: PeerId) -> Self {
        value.0
    }
}
