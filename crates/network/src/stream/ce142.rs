//! Preimage announcement stream.

use crate::{Context, Network};
use quinn::{RecvStream, SendStream};
use score::OpaqueHash;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Send a preimage announcement.
pub async fn send(mut send: SendStream, _recv: RecvStream, request: Request) -> anyhow::Result<()> {
    let mut buf = vec![142];
    buf.extend_from_slice(&codec::encode(&request)?);
    send.write_all(&buf).await?;
    send.finish();
    Ok(())
}

/// Receive a preimage announcement.
///
/// TODO: handle the received preimage.
pub async fn recv<C: Context + Send + Sync + 'static>(
    mut send: SendStream,
    mut recv: RecvStream,
    context: Network<C>,
) -> anyhow::Result<()> {
    let _req: Request = codec::decode(&recv.read_to_end(usize::MAX).await?)?;
    send.finish();
    Ok(())
}

/// A preimage announcement request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Request {
    /// The service id.
    pub service: u32,

    /// The hash.
    pub hash: OpaqueHash,

    /// The preimage length
    pub len: u32,
}
