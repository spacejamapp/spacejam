//! Work report request stream.

use crate::Network;
use quinn::{RecvStream, SendStream};
use score::{service::WorkReport, OpaqueHash};

/// Send a work report request.
#[allow(unused)]
pub async fn send(mut send: SendStream, hash: OpaqueHash) -> anyhow::Result<()> {
    let mut buf = vec![136];
    buf.extend_from_slice(&codec::encode(&hash)?);
    send.write_all(&buf).await?;
    send.finish()?;
    Ok(())
}

impl<C: runtime::Config> Network<C> {
    /// Receive a work report request.
    pub async fn recv_ce136(
        &self,
        mut send: SendStream,
        mut recv: RecvStream,
    ) -> anyhow::Result<()> {
        let _hash: OpaqueHash = codec::decode(&recv.read_to_end(usize::MAX).await?)?;
        // let work_report = runtime.runtime.fetch_work_report(hash)?;
        //
        // TODO: fetch the work report from the storage
        let work_report = WorkReport::default();

        // send the work report
        let mut buf = vec![];
        buf.extend_from_slice(&codec::encode(&work_report)?);
        send.write_all(&buf).await?;
        send.finish()?;
        Ok(())
    }
}
