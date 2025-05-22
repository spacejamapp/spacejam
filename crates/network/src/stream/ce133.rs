//! Work package submission stream.

use crate::Network;
use quinn::{RecvStream, SendStream};
use score::service::WorkPackage;
use serde::{Deserialize, Serialize};

impl<C: runtime::Config> Network<C> {
    /// Receive a work package submission.
    pub async fn recv_ce133(&self, _send: SendStream, _recv: RecvStream) -> anyhow::Result<()> {
        todo!("decode the extrinsic data of work packages.");
    }
}

/// Send a work package submission.
#[allow(unused)]
pub async fn send(mut send: SendStream, request: Request) -> anyhow::Result<()> {
    let mut buf = vec![133];
    buf.extend_from_slice(&codec::encode(&request.message)?);
    send.write_all(&buf).await?;
    send.write_all(&request.extrinsic).await?;
    send.finish()?;
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
