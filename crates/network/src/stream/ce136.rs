//! Work report request stream.

use crate::{Context, Network};
use quinn::{RecvStream, SendStream};
use score::OpaqueHash;

/// Send a work report request.
pub async fn send(mut send: SendStream, recv: RecvStream, hash: OpaqueHash) -> anyhow::Result<()> {
    let mut buf = vec![136];
    buf.extend_from_slice(&codec::encode(&hash)?);
    send.write_all(&buf).await?;
    send.finish();
    Ok(())
}

/// Receive a work report request.
pub async fn recv<C: Context + Send + Sync + 'static>(
    mut send: SendStream,
    mut recv: RecvStream,
    context: Network<C>,
) -> anyhow::Result<()> {
    let hash: OpaqueHash = codec::decode(&recv.read_to_end(usize::MAX).await?)?;
    let work_report = context.context.fetch_work_report(hash)?;

    // send the work report
    let mut buf = vec![];
    buf.extend_from_slice(&codec::encode(&work_report)?);
    send.write_all(&buf).await?;
    send.finish();
    Ok(())
}
