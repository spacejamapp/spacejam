//! Shard distribution stream.

use crate::Network;
use quinn::{RecvStream, SendStream};
use score::{Ed25519Signature, OpaqueHash};
use serde::{Deserialize, Serialize};

impl<C: runtime::Config> Network<C> {
    /// Receive a shard distribution.
    pub async fn recv_ce141(
        &self,
        mut send: SendStream,
        mut recv: RecvStream,
    ) -> anyhow::Result<()> {
        let _request: Request = codec::decode(&recv.read_to_end(usize::MAX).await?)?;
        send.finish()?;
        Ok(())
    }
}

/// Send a shard distribution.
#[allow(unused)]
pub async fn send(mut send: SendStream, request: Request) -> anyhow::Result<()> {
    let mut buf = vec![141];
    buf.extend_from_slice(&codec::encode(&request)?);
    send.write_all(&buf).await?;
    send.finish()?;
    Ok(())
}

/// An assurance distribution request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Request {
    /// The header hash.
    pub hash: OpaqueHash,

    /// The bitfield.
    #[serde(with = "codec::bytes")]
    pub bitfield: [u8; 43],

    /// The signature.
    #[serde(with = "codec::bytes")]
    pub signature: Ed25519Signature,
}
