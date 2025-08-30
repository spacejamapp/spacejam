//! Spacejam's SAFRole prototype

pub use error::{Error, Result};
use score::{
    extrinsic::{
        ticket::{TicketBody, TicketsExtrinsic, TicketsOrKeys},
        TicketsAccumulator,
    },
    safrole::{Safrole, ValidatorData, Validators, ValidatorsData},
    BandersnatchRingCommitment, Ed25519Public, OpaqueHash,
};
use std::time::Instant;

pub mod error;
mod verify;

/// (η') Updates the entropy accumulator.
///
/// graypaper reference: 6.4
pub fn eta(new_epoch: bool, eta: &[OpaqueHash; 4], entropy: OpaqueHash) -> [OpaqueHash; 4] {
    let mut next = *eta;

    // (6.23) eta'_e = H(eta_e || eta'_(e-1))
    if new_epoch {
        let historical_eta = eta;
        next[1..].copy_from_slice(&historical_eta[..3]);
    }

    // (6.22) eta'_0 = H(eta_0 || Y(H_v))
    let eta_0 = crypto::blake2b(&[eta[0], entropy].concat());
    next[0] = eta_0;
    next
}

/// (ι', κ', λ') Returns the next state of validators.
pub fn validators(new_epoch: bool, next: &ValidatorsData, validators: &Validators) -> Validators {
    let mut validators = *validators;
    if !new_epoch {
        return validators;
    }

    validators.previous = validators.previous(new_epoch);
    validators.current = validators.current(new_epoch, next);
    validators
}

/// (γ') Enacts an epoch change and updates the entropy accumulator.
pub async fn safrole(
    tau: u32,
    slot: u32,
    entropy: [OpaqueHash; 4],
    offenders: &[Ed25519Public],
    safrole: &Safrole,
    validators: &Validators,
    tickets: &TicketsExtrinsic,
) -> Result<Safrole> {
    if slot <= tau && tau != 0 {
        tracing::error!("slot {} is less than tau {}", slot, tau);
        return Err(Error::BadSlot);
    }

    let slot_phase = slot % score::EPOCH_LENGTH;
    if slot_phase >= score::TICKET_SUBMISSION_PERIOD && !tickets.is_empty() {
        return Err(Error::UnexpectedTicket);
    }

    let epoch = slot / score::EPOCH_LENGTH;
    let new_epoch: bool = epoch > (tau / score::EPOCH_LENGTH);
    let mut safrole = safrole.clone();
    safrole.series = self::sealing_key_series(tau, slot, entropy, &safrole, &validators.current);
    safrole.validators = safrole.next(new_epoch, &validators.drawn, offenders);

    // Process accumulator and ring commitment in parallel
    tracing::info!("> accumulating and committing tickets...");
    let (accumulator, commitment) = tokio::join!(
        async {
            let now = Instant::now();
            let accumulator = self::accumulator(
                new_epoch,
                &safrole.accumulator,
                entropy,
                &safrole.validators,
                tickets,
            )
            .await;
            tracing::info!("  accumulator time: {}ms", now.elapsed().as_millis());
            accumulator
        },
        async {
            let now = Instant::now();
            let commitment = self::ring_commitment(&safrole, new_epoch).await;
            tracing::info!("  ring_commitment time: {}ms", now.elapsed().as_millis());
            commitment
        }
    );

    safrole.accumulator = accumulator?;
    safrole.ring_commitment = commitment;
    Ok(safrole)
}

/// (γ_a') Verifies tickets and updates the accumulator according to graypaper section 6.7.
///
/// NOTE: gamma_k has already been updated at this point
pub async fn accumulator(
    new_epoch: bool,
    accumulator: &TicketsAccumulator,
    entropy: [OpaqueHash; 4],
    next: &[ValidatorData],
    tickets: &TicketsExtrinsic,
) -> Result<TicketsAccumulator> {
    let mut new_tickets = Vec::new();
    if !tickets.is_empty() {
        new_tickets = self::verify::tickets(entropy, next, tickets).await?;
    }

    // update the accumulator
    let mut accumulator = accumulator.clone();
    if new_epoch {
        // Clear the accumulator if we're starting a new epoch: 6.34
        accumulator.clear();
        accumulator = new_tickets;
    } else {
        // or check for duplicates and create merged set of tickets
        // (formula 6.35: n ∪ γ_a)
        if accumulator.iter().any(|t| new_tickets.contains(t)) {
            return Err(Error::DuplicateTicket);
        }
        accumulator.extend(new_tickets);
    };

    // Sort by identifier
    //
    // Take only the first E tickets (formula 6.35: truncate to E)
    accumulator.sort_by(|a, b| a.id.cmp(&b.id));
    accumulator.truncate(score::EPOCH_LENGTH as usize);
    Ok(accumulator)
}

/// (γ_s') Updates the sealing key series according to graypaper formula 6.24.
pub fn sealing_key_series(
    tau: u32,
    slot: u32,
    entropy: [OpaqueHash; 4],
    safrole: &Safrole,
    curr_validators: &[ValidatorData],
) -> TicketsOrKeys {
    let mut next = safrole.series.clone();
    let curr_epoch = slot / score::EPOCH_LENGTH;
    let prev_epoch = tau / score::EPOCH_LENGTH;
    let prev_slot_phase = tau % score::EPOCH_LENGTH;
    if curr_epoch == prev_epoch {
        return next;
    }

    // FIXME: should be curr_epoch > prev_epoch
    if curr_epoch == prev_epoch + 1
        && prev_slot_phase >= score::TICKET_SUBMISSION_PERIOD
        && safrole.accumulator.len() == score::EPOCH_LENGTH as usize
    {
        next = TicketsOrKeys::Tickets(TicketBody::sequence(&safrole.accumulator));
    } else {
        next = TicketsOrKeys::fallback(
            curr_validators.iter().map(|v| v.bandersnatch).collect(),
            entropy[2],
        );
    }

    next
}

/// (γ_z') Returns the bandersnatch ring commitment.
pub async fn ring_commitment(safrole: &Safrole, new_epoch: bool) -> BandersnatchRingCommitment {
    if !new_epoch {
        return safrole.ring_commitment;
    }

    let keys = safrole
        .validators
        .iter()
        .map(|validator| validator.bandersnatch)
        .collect::<Vec<_>>();
    crypto::ring::commitment(keys)
}
