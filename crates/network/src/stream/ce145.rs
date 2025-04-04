//! Judgement publication stream.

use crate::Network;
use quinn::{RecvStream, SendStream};
use score::{Ed25519Signature, OpaqueHash};
use serde::{Deserialize, Serialize};

/// Send judgement publication.
pub async fn send(
    mut send: SendStream,
    recv: RecvStream,
    judgement: Judgement,
) -> anyhow::Result<()> {
    let mut buf = vec![145];
    buf.extend_from_slice(&codec::encode(&judgement)?);
    send.write_all(&buf).await?;
    send.finish();
    Ok(())
}

/// Receive judgement publication.
///
/// TODO: handle the judgement
pub async fn recv<C: runtime::Config>(
    mut send: SendStream,
    mut recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    let judgement: Judgement = codec::decode(&recv.read_to_end(usize::MAX).await?)?;
    send.finish();
    Ok(())
}

/// Judgement to announce
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Judgement {
    /// epoch index
    pub epoch: u32,

    /// validator index
    pub validator: u32,

    /// validity of the judgement
    pub validity: u8,

    /// the hash of the work report
    pub hash: OpaqueHash,

    /// the signature
    #[serde(with = "codec::bytes")]
    pub signature: Ed25519Signature,
}
