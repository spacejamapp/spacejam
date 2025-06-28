//! Safrole ticket distribution stream (second step).

use crate::{
    stream::{
        ce131,
        ext::{Read, Write},
    },
    Network,
};
pub use ce131::Request;
use quinn::{RecvStream, SendStream, VarInt};
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
        let epoch = block::timeslot() / score::EPOCH_LENGTH;
        tracing::trace!(
            "ticket#{}@{} for epoch: {}",
            request.ticket.attempt,
            hex::encode(&request.ticket.signature[..3]),
            request.epoch,
        );

        if request.epoch == epoch {
            let ticket = self.runtime.verify_ticket(request.ticket).await?;
            self.insert_ticket(epoch, ticket).await?;
        }

        send.stopped().await?;
        send.finish()?;
        recv.stop(VarInt::from_u32(0))?;
        Ok(())
    }
}

/// Send a safrole ticket distribution.
#[tracing::instrument(skip_all, name = "ce132::send", parent = None)]
pub async fn send(
    mut send: SendStream,
    mut recv: RecvStream,
    request: Request,
) -> anyhow::Result<()> {
    send.write(&[132]).await?;

    // 1. send the request
    request.write(&mut send).await?;

    // 2. finish sending and wait for the response to be fully received
    send.stopped().await?;
    send.finish()?;
    recv.stop(VarInt::from_u32(0))?;
    Ok(())
}
