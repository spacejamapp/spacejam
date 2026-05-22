//! Verification utilities for tickets

use crate::tx::ticket::{Error, lazy};
use score::{
    BandersnatchPublic, OpaqueHash,
    extrinsic::{TicketBody, TicketsAccumulator, TicketsExtrinsic},
};

/// Verify tickets
pub fn tickets(
    entropy: [OpaqueHash; 4],
    next: &Vec<BandersnatchPublic>,
    tickets: &TicketsExtrinsic,
) -> Result<TicketsAccumulator, Error> {
    // 1. Verify ticket attempts upfront (6.29)
    for envelope in tickets.iter() {
        if envelope.attempt >= score::TICKET_ENTRIES_PER_VALIDATOR as u8 {
            return Err(Error::BadTicketAttempt);
        }
    }

    // 2. Batch-verify ring VRF signatures, harvesting per-ticket ids in order
    let messages: Vec<Vec<u8>> = tickets
        .iter()
        .map(|e| TicketBody::message(e.attempt, &entropy[2]))
        .collect();
    let verifier = lazy::verifier(next);
    let ids = verifier
        .ring_vrf_verify_batch(
            messages
                .iter()
                .zip(tickets.iter())
                .map(|(msg, e)| (msg.as_slice(), [].as_slice(), e.signature.as_slice())),
        )
        .map_err(|e| {
            tracing::trace!("failed to batch-verify ring VRF signatures: {:?}", e);
            Error::BadTicketProof
        })?;

    let new_tickets: Vec<TicketBody> = ids
        .into_iter()
        .zip(tickets.iter())
        .map(|(id, envelope)| TicketBody {
            id,
            attempt: envelope.attempt,
        })
        .collect();

    // 3. Strictly ascending by id, no duplicates (6.32 & 6.33)
    for pair in new_tickets.windows(2) {
        match pair[0].id.cmp(&pair[1].id) {
            std::cmp::Ordering::Less => {}
            std::cmp::Ordering::Equal => return Err(Error::DuplicateTicket),
            std::cmp::Ordering::Greater => return Err(Error::BadTicketOrder),
        }
    }

    Ok(new_tickets)
}
