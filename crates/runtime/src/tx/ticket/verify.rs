//! Verification utilities for tickets

use crate::tx::ticket::Error;
use score::{
    extrinsic::{TicketBody, TicketEnvelope, TicketsAccumulator, TicketsExtrinsic},
    safrole::ValidatorData,
    OpaqueHash,
};
use std::{collections::BTreeMap, sync::Arc, time::Instant};
use tokio::task::JoinSet;

/// Verify tickets
pub async fn tickets(
    entropy: [OpaqueHash; 4],
    next: &[ValidatorData],
    tickets: &TicketsExtrinsic,
) -> Result<TicketsAccumulator, Error> {
    let now = Instant::now();
    let verifier = Arc::new(crypto::ring::verifier(
        next.iter().map(|v| v.bandersnatch).collect(),
    ));
    tracing::info!(
        "    setting up verifier time: {}ms",
        now.elapsed().as_millis()
    );

    // process verification in parallel
    let now = Instant::now();
    let mut queue = JoinSet::new();
    for (index, envelope) in tickets.iter().cloned().enumerate() {
        let verifier = verifier.clone();
        queue.spawn_blocking(move || self::ticket(index, envelope, entropy, verifier));
    }

    let mut ordered_tickets = BTreeMap::new();
    while let Some(ticket) = queue.join_next().await {
        let (index, ticket) = ticket.map_err(|_| Error::Reserved)??;
        ordered_tickets.insert(index, ticket);
    }

    // Check for bad order: 6.32 & 6.33
    let new_tickets = ordered_tickets.into_values().collect::<Vec<_>>();
    tracing::info!(
        "    verifying tickets time: {}ms, tickets count: {}",
        now.elapsed().as_millis(),
        new_tickets.len()
    );
    let mut sorted_new_tickets = new_tickets.clone();
    sorted_new_tickets.sort_by(|a, b| a.id.cmp(&b.id));
    if sorted_new_tickets != new_tickets {
        return Err(Error::BadTicketOrder);
    }

    Ok(new_tickets)
}

/// Verify a single ticket
fn ticket(
    index: usize,
    envelope: TicketEnvelope,
    entropy: [OpaqueHash; 4],
    verifier: Arc<crypto::vrf::Verifier>,
) -> Result<(usize, TicketBody), Error> {
    // 1. Verify ticket attempt (6.29)
    if envelope.attempt > score::TICKET_ENTRIES_PER_VALIDATOR as u8 {
        return Err(Error::BadTicketAttempt);
    }

    // 2. Verify ring VRF signature and get ticket identifier
    let id = verifier
        .ring_vrf_verify(
            &TicketBody::message(envelope.attempt, &entropy[2]),
            &[],
            &envelope.signature,
        )
        .map_err(|e| {
            tracing::error!("failed to verify ring VRF signature: {:?}", e);
            Error::BadTicketProof
        })?;

    // 3. Store ticket for accumulation
    Ok((
        index,
        TicketBody {
            id,
            attempt: envelope.attempt,
        },
    ))
}
