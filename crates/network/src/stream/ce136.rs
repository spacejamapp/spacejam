//! Work report request stream.

use crate::Network;
use quinn::{RecvStream, SendStream};
use score::{service::WorkReport, OpaqueHash};

/// Send a work report request.
pub async fn send(mut send: SendStream, recv: RecvStream, hash: OpaqueHash) -> anyhow::Result<()> {
    let mut buf = vec![136];
    buf.extend_from_slice(&codec::encode(&hash)?);
    send.write_all(&buf).await?;
    send.finish();
    Ok(())
}

/// Receive a work report request.
pub async fn recv<C: runtime::Config>(
    mut send: SendStream,
    mut recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    let hash: OpaqueHash = codec::decode(&recv.read_to_end(usize::MAX).await?)?;
    // let work_report = runtime.runtime.fetch_work_report(hash)?;
    //
    // TODO: fetch the work report from the storage
    let work_report = WorkReport::default();

    // send the work report
    let mut buf = vec![];
    buf.extend_from_slice(&codec::encode(&work_report)?);
    send.write_all(&buf).await?;
    send.finish();
    Ok(())
}
