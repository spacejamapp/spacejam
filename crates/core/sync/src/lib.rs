//! Block sync validation

use anyhow::Result;
use score::{validator::Validator, Block, State};

pub mod assurance;
pub mod dispute;
pub mod guarantee;
pub mod preimage;
pub mod statistic;
pub mod ticket;

/// Transit state with new block
pub fn transit(block: &Block, mut state: State, validator: impl Validator) -> Result<State> {
    let epoch = block.header.slot / score::EPOCH_LENGTH;
    let new_epoch: bool = epoch > (state.timeslot / score::EPOCH_LENGTH);

    // The first round computation
    {
        // (η') Update entropy (6.22)
        let entropy = validator.entropy(state.entropy[0], &block.header.entropy_source)?;
        state.entropy = ticket::eta(new_epoch, &state.entropy, entropy);

        // (λ') Update validator state (6.13)
        state.validators.previous = state.validators.previous(new_epoch);

        // (ψ') Update disputes and get marks
        let (disputes, marks) = crate::dispute::disputes(
            state.timeslot,
            &state.validators.current,
            &state.validators.previous,
            &state.disputes,
            &block.extrinsic.disputes,
        )?;
        state.disputes = disputes;

        // (ρ†) Update availability assignments based on verdicts (V) (10.15)
        state.reports = dispute::reports(&marks, &state.reports);

        // (ρ‡) Update availability assignments based on assurances (11.17)
        state.reports = crate::assurance::reports(block.header.slot, state.reports);

        // (ρ') Update availability assignments based on guarantees (11.43)
        state.reports = guarantee::reports(
            block.header.slot,
            &state.reports,
            &block.extrinsic.guarantees,
        )?;
    }

    // Round 2 computation
    {
        // (κ') Update current validators (6.13)
        state.validators.current = state
            .validators
            .current(new_epoch, &state.safrole.validators);

        // (W) the sequence of new available work reports (11.16)
        //
        // TODO: not sure why we still have mutation for `state.reports` here.
        let (_assignments, _reports) = crate::assurance::available(
            &state.reports,
            &state.validators.current,
            block.header.slot,
            block.header.parent,
            &block.extrinsic.assurances,
        )?;

        // (W*) The sequence of accumulatable work-reports (12.9)
        //
        // TODO: not yet implemented.
    }

    // Round 3 computation
    {
        // (γ') Update the sealing-key series (12.10)
        state.safrole = ticket::safrole(
            state.timeslot,
            block.header.slot,
            state.entropy,
            &state.disputes.offenders,
            &state.safrole,
            &state.validators,
            &block.extrinsic.tickets,
        )?;

        // (π') Update the statistic
        // state.statistics =
        //     statistic::update(&state.statistics, &state.validators.current, &state.safrole)?;

        // TODO: ACCUMULATION 12
    }

    Ok(state)
}
