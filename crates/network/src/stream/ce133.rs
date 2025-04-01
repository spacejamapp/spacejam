//! Work package submission stream.

use crate::Network;
use quinn::{RecvStream, SendStream};
use score::{extrinsic::GuaranteesExtrinsic, service::WorkPackage};
use serde::{Deserialize, Serialize};

/// Send a work package submission.
pub async fn send(
    mut send: SendStream,
    mut recv: RecvStream,
    request: Request,
) -> anyhow::Result<()> {
    let mut buf = vec![133];
    buf.extend_from_slice(&codec::encode(&request.message)?);
    send.write_all(&buf).await?;
    send.write_all(&request.extrinsic).await?;
    send.finish();
    Ok(())
}

/// Receive a work package submission.
pub async fn recv<C: runtime::Config>(
    send: SendStream,
    recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    todo!("decode the extrinsic data of work packages.");
    Ok(())
}

/// A work package submission request.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct Request {
    /// The message.
    pub message: Message,

    /// The guarantees.
    pub extrinsic: Vec<u8>,
}

/// A work package submission message.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct Message {
    /// The core index.
    pub core_index: u32,

    /// The work package.
    pub package: WorkPackage,
}
