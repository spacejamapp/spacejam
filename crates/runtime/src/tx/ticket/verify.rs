//! Verification utilities for tickets

use crate::tx::ticket::{Error, lazy};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use score::{
    BandersnatchPublic, OpaqueHash,
    extrinsic::{TicketBody, TicketEnvelope, TicketsAccumulator, TicketsExtrinsic},
};
use std::{collections::BTreeMap, sync::Arc};

/// Verify tickets
pub fn tickets(
    entropy: [OpaqueHash; 4],
    next: &Vec<BandersnatchPublic>,
    tickets: &TicketsExtrinsic,
) -> Result<TicketsAccumulator, Error> {
    let verifier = lazy::verifier(next);
    let verified = tickets
        .par_iter()
        .enumerate()
        .map(|(index, envelope)| self::ticket(index, envelope.clone(), entropy, verifier.clone()))
        .collect::<Result<BTreeMap<usize, TicketBody>, Error>>()?;

    // Check for bad order: 6.32 & 6.33
    let new_tickets = verified.into_values().collect::<Vec<_>>();
    let mut sorted = new_tickets.clone();
    sorted.sort_by_key(|a| a.id);
    if sorted != new_tickets {
        return Err(Error::BadTicketOrder);
    }

    Ok(sorted)
}

/// Verify a single ticket
fn ticket(
    index: usize,
    envelope: TicketEnvelope,
    entropy: [OpaqueHash; 4],
    verifier: Arc<crypto::vrf::Verifier>,
) -> Result<(usize, TicketBody), Error> {
    // 1. Verify ticket attempt (6.29)
    if envelope.attempt >= score::TICKET_ENTRIES_PER_VALIDATOR as u8 {
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
