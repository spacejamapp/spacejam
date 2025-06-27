//! Safrole ticket distribution stream (first step).

use crate::{stream::ext::Write, Network};
use quinn::{RecvStream, SendStream};
use score::extrinsic::TicketEnvelope;
use serde::{Deserialize, Serialize};
use std::mem;

impl<C: runtime::Config> Network<C> {
    /// Receive a safrole ticket distribution.
    #[tracing::instrument(skip_all, name = "ce131::recv", parent = None)]
    pub async fn recv_ce131(
        &self,
        mut send: SendStream,
        mut recv: RecvStream,
    ) -> anyhow::Result<()> {
        let size = mem::size_of::<Request>();
        let mut buf = vec![0; size];
        recv.read_exact(&mut buf).await?;

        // TODO: verify the proof, handle the ticket, etc.
        let request: Request = codec::decode(&buf[..])?;
        tracing::info!(
            "received safrole ticket request: for epoch {}",
            request.epoch
        );
        send.finish()?;
        Ok(())
    }
}

/// Send a safrole ticket distribution.
#[allow(unused)]
pub async fn send(mut send: SendStream, _recv: RecvStream, request: Request) -> anyhow::Result<()> {
    send.write(&[132]).await?;
    request.write(&mut send).await?;
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
