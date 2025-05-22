//! Work report distribution stream.

use crate::Network;
use quinn::{RecvStream, SendStream};
use score::{service::WorkReport, Ed25519Signature};
use serde::{Deserialize, Serialize};

impl<C: runtime::Config> Network<C> {
    /// Receive a work report distribution.
    ///
    /// TODO: handle the received work report.
    pub async fn recv_ce135(
        &self,
        mut send: SendStream,
        mut recv: RecvStream,
    ) -> anyhow::Result<()> {
        let _req: Request = codec::decode(&recv.read_to_end(usize::MAX).await?)?;
        send.finish()?;
        Ok(())
    }
}

/// Send a work report distribution.
#[allow(unused)]
pub async fn send(
    mut send: SendStream,
    mut recv: RecvStream,
    request: Request,
) -> anyhow::Result<()> {
    let mut buf = vec![135];
    buf.extend_from_slice(&codec::encode(&request)?);
    send.write_all(&buf).await?;
    send.finish();
    Ok(())
}

/// A work package request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Request {
    /// The work report.
    pub work_report: WorkReport,

    /// The slot.
    pub slot: u32,

    /// The signatures.
    pub signatures: Vec<IndexedSignature>,
}

/// An indexed signature.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct IndexedSignature {
    /// The index.
    pub index: u32,

    /// The signature.
    #[serde(with = "codec::bytes")]
    pub signature: Ed25519Signature,
}
