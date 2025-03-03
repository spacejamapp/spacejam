//! Peer related stuffs

use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
pub use {
    address::Address,
    pool::{Connection, Pool},
    sync::Sync,
};

mod address;
mod pool;
mod sync;

/// Peer ID, also known as the DNS name of the peer.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(String);

impl PeerId {
    /// Create a new peer id.
    pub fn verify(id: &str) -> Result<[u8; 32], anyhow::Error> {
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
                bytes
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("failed to convert peer id bytes to [u8; 32]"))
            })
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for PeerId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for PeerId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let _ = Self::verify(s)?;
        Ok(Self(s.to_string()))
    }
}

impl From<&[u8; 32]> for PeerId {
    fn from(value: &[u8; 32]) -> Self {
        Self(format!(
            "e{}",
            base32::encode(base32::Alphabet::Rfc4648Lower { padding: false }, value)
        ))
    }
}

impl TryFrom<PeerId> for [u8; 32] {
    type Error = anyhow::Error;

    fn try_from(value: PeerId) -> Result<Self, Self::Error> {
        PeerId::verify(&value.0)
    }
}
