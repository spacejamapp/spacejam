//! Shard distribution stream.

use crate::{Context, Network};
use quinn::{RecvStream, SendStream};
use score::{Ed25519Signature, OpaqueHash};
use serde::{Deserialize, Serialize};

/// Send a shard distribution.
pub async fn send(mut send: SendStream, _recv: RecvStream, request: Request) -> anyhow::Result<()> {
    let mut buf = vec![137];
    buf.extend_from_slice(&codec::encode(&request)?);
    send.write_all(&buf).await?;
    send.finish();
    Ok(())
}

/// Receive a shard distribution.
pub async fn recv<C: Context + Send + Sync + 'static>(
    mut send: SendStream,
    mut recv: RecvStream,
    context: Network<C>,
) -> anyhow::Result<()> {
    let request: Request = codec::decode(&recv.read_to_end(usize::MAX).await?)?;
    send.finish();
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
