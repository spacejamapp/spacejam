//! Preimage announcement stream.

use crate::Network;
use quinn::{RecvStream, SendStream};
use score::OpaqueHash;
use serde::{Deserialize, Serialize};

impl<C: runtime::Config> Network<C> {
    /// Receive a preimage announcement.
    ///
    /// TODO: handle the received preimage.
    pub async fn recv_ce142(
        &self,
        mut send: SendStream,
        mut recv: RecvStream,
    ) -> anyhow::Result<()> {
        let _req: Request = codec::decode(&recv.read_to_end(usize::MAX).await?)?;
        send.finish()?;
        Ok(())
    }
}

/// Send a preimage announcement.
#[allow(unused)]
pub async fn send(mut send: SendStream, request: Request) -> anyhow::Result<()> {
    let mut buf = vec![142];
    buf.extend_from_slice(&codec::encode(&request)?);
    send.write_all(&buf).await?;
    send.finish()?;
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
