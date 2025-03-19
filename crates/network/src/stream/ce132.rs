//! Safrole ticket distribution stream (second step).

use crate::{peer::PeerId, stream::ce131, Network};
pub use ce131::Request;
use crypto::vrf::RingVrfSignature;
use quinn::{RecvStream, SendStream};
use score::{block, extrinsic::TicketEnvelope, runtime::Storage, Ed25519Public};
use serde::{Deserialize, Serialize};
use std::mem;

/// Send a safrole ticket distribution.
#[tracing::instrument(skip_all, name = "ce132::send", parent = None)]
pub async fn send(
    mut send: SendStream,
    mut recv: RecvStream,
    request: Request,
) -> anyhow::Result<()> {
    let mut buf = vec![132];
    buf.extend_from_slice(&codec::encode(&request)?);
    send.write_all(&buf).await?;

    // just wait for the response
    let _ = recv.read_to_end(0).await;
    send.finish();
    Ok(())
}

/// Receive a safrole ticket distribution.
#[tracing::instrument(skip_all, name = "ce132::recv", parent = None)]
pub async fn recv<C: score::runtime::Config>(
    mut send: SendStream,
    mut recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    let mut buf = vec![0; 789];
    recv.read_exact(&mut buf).await?;
    send.finish();

    // TODO: verify the proof, handle the ticket, etc.
    let request: Request = codec::decode(&buf[..])?;
    let epoch = block::timeslot()? / score::EPOCH_LENGTH;

    // insert the ticket into the pool if the epoch is present.
    if request.epoch == epoch {
        runtime
            .author()
            .insert_ticket(epoch, request.ticket.clone())
            .await?;
    }

    tracing::trace!(
        "ticket#{} for epoch: {}",
        request.ticket.attempt,
        request.epoch
    );
    Ok(())
}
