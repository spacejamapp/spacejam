//! Safrole ticket distribution stream (first step).

use crate::{stream::ext::Write, Network};
use quinn::{RecvStream, SendStream, VarInt};
use score::{block, extrinsic::TicketEnvelope};
use serde::{Deserialize, Serialize};

impl<C: runtime::Config> Network<C> {
    /// Receive a safrole ticket distribution.
    #[tracing::instrument(skip_all, name = "ce131::recv", parent = None)]
    pub async fn recv_ce131(
        &self,
        mut send: SendStream,
        mut recv: RecvStream,
    ) -> anyhow::Result<()> {
        let mut buf = [0; 4];
        recv.read_exact(&mut buf).await?;
        let length = u32::from_le_bytes(buf);

        let mut buf = vec![0; length as usize];
        recv.read_exact(&mut buf).await?;

        // TODO: verify the proof, handle the ticket, etc.
        let request: Request = codec::decode(&buf)?;
        let epoch = block::timeslot() / score::EPOCH_LENGTH;

        // insert the ticket into the pool if the epoch is present.
        if request.epoch == epoch {
            self.insert_ticket(epoch, request.ticket.clone()).await?;
        }

        tracing::trace!(
            "ticket#{}@{} for epoch: {}",
            request.ticket.attempt,
            hex::encode(&request.ticket.signature[..3]),
            request.epoch,
        );
        send.finish()?;
        recv.stop(VarInt::from_u32(0))?;
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
