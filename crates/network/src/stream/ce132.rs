//! Safrole ticket distribution stream (second step).

use crate::{stream::ce131, Network};
pub use ce131::Request;
use quinn::{RecvStream, SendStream};
use score::block;

use super::ext::Write;

impl<C: runtime::Config> Network<C> {
    /// Receive a safrole ticket distribution.
    #[tracing::instrument(skip_all, name = "ce132::recv", parent = None)]
    pub async fn recv_ce132(
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
            "ticket#{} for epoch: {}",
            request.ticket.attempt,
            request.epoch
        );
        send.finish()?;
        Ok(())
    }
}

/// Send a safrole ticket distribution.
#[tracing::instrument(skip_all, name = "ce132::send", parent = None)]
pub async fn send(mut send: SendStream, _recv: RecvStream, request: Request) -> anyhow::Result<()> {
    send.write(&[132]).await?;

    // 1. send the request
    request.write(&mut send).await?;

    // 2. finish sending and wait for the response to be fully received
    send.finish()?;

    Ok(())
}
