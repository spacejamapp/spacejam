//! Safrole ticket distribution stream (second step).

use crate::{
    stream::{
        ce131,
        ext::{Read, Write},
    },
    Network,
};
pub use ce131::Request;
use quinn::{RecvStream, SendStream};
use score::block;

impl<C: runtime::Config> Network<C> {
    /// Receive a safrole ticket distribution.
    #[tracing::instrument(skip_all, name = "ce132::recv", parent = None)]
    pub async fn recv_ce132(
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
            tracing::warn!(
                "received out-of-epoch ticket: epoch {} != {} (current) or {} (next), ignoring",
                request.epoch,
                current_epoch,
                current_epoch + 1
            );
            send.finish()?;
            return Ok(());
        }

        let attempt = request.ticket.attempt;
        if request.epoch == current_epoch || request.epoch == current_epoch + 1 {
            let ticket = self.runtime.verify_ticket(request.ticket).await?;
            self.insert_ticket(request.epoch, ticket).await?;

            tracing::debug!(
                "accepted ticket#{} for epoch {}, stored in pool",
                attempt,
                request.epoch
            );
        } else {
            tracing::debug!(
                "ignoring ticket#{} for wrong epoch: {} (current: {})",
                attempt,
                request.epoch,
                current_epoch
            );
        }

        send.finish()?;
        Ok(())
    }
}

/// Send a safrole ticket distribution.
#[tracing::instrument(skip_all, name = "ce132::send", parent = None)]
pub async fn send(mut send: SendStream, _recv: RecvStream, request: Request) -> anyhow::Result<()> {
    send.write(&[132]).await?;
    request.write(&mut send).await?;
    send.finish()?;
    Ok(())
}
