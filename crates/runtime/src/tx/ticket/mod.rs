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
    if !new_epoch {
        return validators.clone();
    }

    Validators {
        previous: validators.previous(new_epoch).clone(),
        current: validators.current(new_epoch, next).clone(),
        drawn: validators.drawn.clone(),
    }
}

/// (γ') Enacts an epoch change and updates the entropy accumulator.
pub fn safrole(
    tau: u32,
    slot: u32,
    entropy: [OpaqueHash; 4],
    offenders: &[Ed25519Public],
    mut safrole: Safrole,
    validators: &Validators,
    tickets: &TicketsExtrinsic,
) -> Result<Safrole> {
    if slot <= tau && tau != 0 {
        tracing::error!("slot {} is less than tau {}", slot, tau);
        return Err(Error::BadSlot);
    }

    let slot_phase = slot % score::EPOCH_LENGTH;
    if slot_phase >= score::TICKET_SUBMISSION_PERIOD {
        if !tickets.is_empty() {
            return Err(Error::UnexpectedTicket);
        }
    } else if tickets.len() > score::MAX_TICKETS_PER_EXTRINSIC as usize {
        return Err(Error::TooManyTickets);
    }

    let epoch = tau / score::EPOCH_LENGTH;
    let next_epoch = slot / score::EPOCH_LENGTH;
    let new_epoch: bool = next_epoch > epoch;
    if let Some(series) =
        self::sealing_key_series(tau, slot, entropy, &safrole, &validators.current)
    {
        safrole.series = series;
    }
    if new_epoch {
        let next = safrole.next(&validators.drawn, offenders);
        if next != safrole.validators {
            safrole.ring_commitment = lazy::commitment(&next.bandersnatch());
            safrole.validators = next;
        }
    }

    // Process accumulator and ring commitment in parallel
    let acc = std::mem::take(&mut safrole.accumulator);
    safrole.accumulator = self::accumulator(
        new_epoch,
        acc,
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
    mut accumulator: TicketsAccumulator,
    entropy: [OpaqueHash; 4],
    next: &Vec<BandersnatchPublic>,
    tickets: &TicketsExtrinsic,
) -> Result<TicketsAccumulator> {
    let mut new_tickets = Vec::new();
    if !tickets.is_empty() {
        new_tickets = self::verify::tickets(entropy, next, tickets)?;
    }

    // Snapshot submitted ids for the n ⊆ γ_a' check below
    let submitted_ids: Vec<OpaqueHash> = new_tickets.iter().map(|t| t.id).collect();

    // update the accumulator
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
    accumulator.sort_by_key(|a| a.id);
    accumulator.truncate(score::EPOCH_LENGTH as usize);

    // (n ⊆ γ_a') Every submitted ticket must survive into the posterior
    // accumulator; a submission that gets truncated out is "useless".
    if submitted_ids
        .iter()
        .any(|id| accumulator.binary_search_by_key(id, |t| t.id).is_err())
    {
        return Err(Error::UselessTicket);
    }

    Ok(accumulator)
}

/// (γ_s') Updates the sealing key series according to graypaper formula 6.24.
/// Returns `None` when the series is unchanged (same epoch).
pub fn sealing_key_series(
    tau: u32,
    slot: u32,
    entropy: [OpaqueHash; 4],
    safrole: &Safrole,
    curr_validators: &[ValidatorData],
) -> Option<TicketsOrKeys> {
    let curr_epoch = slot / score::EPOCH_LENGTH;
    let prev_epoch = tau / score::EPOCH_LENGTH;
    let prev_slot_phase = tau % score::EPOCH_LENGTH;
    if curr_epoch == prev_epoch {
        return None;
    }

    // FIXME: should be curr_epoch > prev_epoch
    let next = if curr_epoch == prev_epoch + 1
        && prev_slot_phase >= score::TICKET_SUBMISSION_PERIOD
        && safrole.accumulator.len() == score::EPOCH_LENGTH as usize
    {
        TicketsOrKeys::Tickets(TicketBody::sequence(&safrole.accumulator))
    } else {
        TicketsOrKeys::fallback(&curr_validators.bandersnatch(), entropy[2])
    };
    Some(next)
}
