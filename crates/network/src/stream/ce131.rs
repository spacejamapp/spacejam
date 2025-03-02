//! Safrole ticket distribution stream (first step).

use crate::Network;
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
    let mut buf = vec![131];
    buf.extend_from_slice(&codec::encode(&request)?);
    send.write_all(&buf).await?;
    send.finish();
    Ok(())
}

/// Receive a safrole ticket distribution.
pub async fn recv<C: score::runtime::Config>(
    mut send: SendStream,
    mut recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    let size = mem::size_of::<Request>();
    let mut buf = vec![0; size];
    recv.read_exact(&mut buf).await?;

    // TODO: verify the proof, handle the ticket, etc.
    let _request: Request = codec::decode(&buf[..])?;
    send.finish();
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
