//! Safrole ticket distribution stream (first step).

use crate::{
    stream::ext::{Read, Write},
    Network,
};
use quinn::{RecvStream, SendStream};
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
        let current_slot = block::timeslot();
        let current_epoch = current_slot / score::EPOCH_LENGTH;

        tracing::trace!(
            "ticket#{}@{} for epoch: {} (current epoch: {})",
            request.ticket.attempt,
            hex::encode(&request.ticket.signature[..3]),
            request.epoch,
            current_epoch
        );

        if request.epoch != current_epoch + 1 && request.epoch != current_epoch {
            send.finish()?; // Finish stream before error
            anyhow::bail!(
                    "received invalid ticket: epoch mismatch: {} != {} (current) or {} (next), rejecting out-of-epoch ticket",
                    request.epoch,
                    current_epoch,
                    current_epoch + 1
                );
        }

        let attempt = request.ticket.attempt;
        let ticket = self.runtime.verify_ticket(request.ticket).await?;
        let submission = ticket.submission();
        let validators = self.runtime.grid().await.next;
        let validator = validators[submission];
        let me = self.me();
        if validator.bandersnatch != me {
            send.finish()?; // Finish stream before error
            anyhow::bail!(
                "received invalid ticket: not the proxy validator, expected: {}, this: {:?}",
                submission,
                validators.iter().position(|v| v.ed25519 == me),
            );
        }

        self.insert_ticket(request.epoch, ticket.clone()).await?;
        self.runtime
            .tickets
            .lock()
            .await
            .push((request.epoch, ticket.envelope));

        tracing::debug!(
            "accepted ticket#{} for epoch {}, stored in pool",
            attempt,
            request.epoch
        );

        send.finish()?;
        Ok(())
    }
}

/// Submit a safrole ticket
pub async fn send(mut send: SendStream, _recv: RecvStream, request: Request) -> anyhow::Result<()> {
    send.write(&[131]).await?;
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
