//! Safrole ticket distribution stream (first step).

use crate::Network;
use quinn::{RecvStream, SendStream};
use score::extrinsic::TicketEnvelope;
use serde::{Deserialize, Serialize};
use std::mem;

impl<C: runtime::Config> Network<C> {
    /// Receive a safrole ticket distribution.
    pub async fn recv_ce131(
        &self,
        mut send: SendStream,
        mut recv: RecvStream,
    ) -> anyhow::Result<()> {
        let size = mem::size_of::<Request>();
        let mut buf = vec![0; size];
        recv.read_exact(&mut buf).await?;

        // TODO: verify the proof, handle the ticket, etc.
        let _request: Request = codec::decode(&buf[..])?;
        send.finish()?;
        Ok(())
    }
}

/// Send a safrole ticket distribution.
#[allow(unused)]
pub async fn send(mut send: SendStream, _recv: RecvStream, request: Request) -> anyhow::Result<()> {
    let mut buf = vec![131];
    buf.extend_from_slice(&codec::encode(&request)?);
    send.write_all(&buf).await?;
    send.finish()?;
    Ok(())
}

/// A safrole ticket request.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct Request {
    /// The epoch index
    pub epoch: u32,

    /// The ticket
    pub ticket: TicketEnvelope,
}
