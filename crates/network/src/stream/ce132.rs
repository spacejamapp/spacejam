//! Safrole ticket distribution stream (second step).

use crate::{stream::ce131, Network};
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
        let mut buf = vec![0; 789];
        recv.read_exact(&mut buf).await?;
        send.finish()?;

        // TODO: verify the proof, handle the ticket, etc.
        let request: Request = codec::decode(&buf[..])?;
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
    let encoded = codec::encode(&request)?;
    send.write(&encoded.len().to_le_bytes()).await?;
    send.write(&encoded).await?;

    // 2. just wait for the response
    recv.read_to_end(0).await?;
    send.finish()?;
    Ok(())
}
