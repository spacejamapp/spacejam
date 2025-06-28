//! Safrole ticket distribution stream (first step).

use crate::{
    stream::ext::{Read, Write},
    Network,
};
use quinn::{RecvStream, SendStream, VarInt};
use score::{block, extrinsic::TicketEnvelope};
use serde::{Deserialize, Serialize};

impl<C: runtime::Config> Network<C> {
    /// Receive a safrole ticket distribution.
    ///
    /// FIXME: cache the ticket and send them after the threshold.
    #[tracing::instrument(skip_all, name = "ce131::recv", parent = None)]
    pub async fn recv_ce131(
        &self,
        mut send: SendStream,
        mut recv: RecvStream,
    ) -> anyhow::Result<()> {
        let request = Request::read(&mut recv).await?;
        let epoch = block::timeslot() / score::EPOCH_LENGTH;

        tracing::trace!(
            "ticket#{}@{} for epoch: {}",
            request.ticket.attempt,
            hex::encode(&request.ticket.signature[..3]),
            request.epoch,
        );

        // check if the ticket is valid.
        let ticket = self.runtime.verify_ticket(request.ticket).await?;
        let submission = ticket.submission();
        let validators = self.grandpa().await.grid.curr;
        let validator = validators[submission];
        if validator.ed25519 != self.me() {
            anyhow::bail!("received invalid ticket: not the proxy validator");
        }

        if request.epoch != epoch {
            anyhow::bail!(
                "received invalid ticket: epoch mismatch: {} != {}, FIXME: detect epoch from best head",
                request.epoch,
                epoch
            );
        }

        self.insert_ticket(epoch, ticket.clone()).await?;
        self.runtime
            .tickets
            .lock()
            .await
            .push((epoch, ticket.envelope));
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
