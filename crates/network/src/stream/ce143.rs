//! Preimage request stream.

use crate::{Context, Network};
use quinn::{RecvStream, SendStream};
use score::OpaqueHash;
use std::sync::Arc;

/// Send a preimage request.
pub async fn send(
    mut send: SendStream,
    recv: RecvStream,
    request: OpaqueHash,
) -> anyhow::Result<()> {
    let mut buf = vec![143];
    buf.extend_from_slice(&codec::encode(&request)?);
    send.write_all(&buf).await?;
    send.finish();
    Ok(())
}

/// Receive a preimage request.
pub async fn recv<C: Context + Send + Sync + 'static>(
    mut send: SendStream,
    mut recv: RecvStream,
    context: Network<C>,
) -> anyhow::Result<()> {
    let mut hash = [0; 32];
    recv.read_exact(&mut hash).await?;

    // fetch the preimage
    let preimage = context.context.fetch_preimage(hash)?;
    send.write_all(&preimage).await?;
    send.finish();
    Ok(())
}
