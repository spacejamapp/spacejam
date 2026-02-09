//! Spacejam's SAFRole prototype

pub use error::{Error, Result};
use score::{
    BandersnatchPublic, Ed25519Public, OpaqueHash,
    extrinsic::{
        TicketsAccumulator,
        ticket::{TicketBody, TicketsExtrinsic, TicketsOrKeys},
    },
    safrole::{Safrole, ValidatorData, ValidatorIter, Validators, ValidatorsData},
};

pub mod error;
pub mod lazy;
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
pub fn safrole(
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

    let epoch = tau / score::EPOCH_LENGTH;
    let next_epoch = slot / score::EPOCH_LENGTH;
    let new_epoch: bool = next_epoch > epoch;
    let mut safrole = safrole.clone();
    safrole.series = self::sealing_key_series(tau, slot, entropy, &safrole, &validators.current);
    if new_epoch {
        let next = safrole.next(&validators.drawn, offenders);
        if next != safrole.validators {
            safrole.validators = next;
            safrole.ring_commitment = lazy::commitment(&next.bandersnatch());
        }
    }

    // Process accumulator and ring commitment in parallel
    safrole.accumulator = self::accumulator(
        new_epoch,
        &safrole.accumulator,
        entropy,
        &safrole.validators.bandersnatch(),
        tickets,
    )?;

    Ok(safrole)
}

/// (γ_a') Verifies tickets and updates the accumulator according to graypaper section 6.7.
///
/// NOTE: gamma_k has already been updated at this point
pub fn accumulator(
    new_epoch: bool,
    accumulator: &TicketsAccumulator,
    entropy: [OpaqueHash; 4],
    next: &Vec<BandersnatchPublic>,
    tickets: &TicketsExtrinsic,
) -> Result<TicketsAccumulator> {
    let mut new_tickets = Vec::new();
    if !tickets.is_empty() {
        new_tickets = self::verify::tickets(entropy, next, tickets)?;
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
        next = TicketsOrKeys::fallback(curr_validators.bandersnatch(), entropy[2]);
    }

    next
}
