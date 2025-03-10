//! Safrole ticket distribution stream (second step).

use crate::{stream::ce131, Network};
pub use ce131::Request;
use crypto::vrf::RingVrfSignature;
use quinn::{RecvStream, SendStream};
use score::extrinsic::TicketEnvelope;
use serde::{Deserialize, Serialize};
use std::mem;

/// Send a safrole ticket distribution.
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
    runtime.expool.insert_ticket(request.epoch, request.ticket);
    Ok(())
}
