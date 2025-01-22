//! Block sync validation

use anyhow::Result;
use score::{block::History, validator::Validator, work::WorkReport, Block, OpaqueHash, State};

pub mod assurance;
pub mod dispute;
pub mod guarantee;
pub mod preimage;
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
    let available = {
        // (κ') Update current validators (6.13)
        state.validators.current = state
            .validators
            .current(new_epoch, &state.safrole.validators);

        // (W*) the sequence of new available work reports (11.16)
        crate::assurance::available(
            &state.reports,
            &state.validators.current,
            block.header.slot,
            block.header.parent,
            &block.extrinsic.assurances,
        )?
    };

    // Round 3 computation
    let root = {
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
        state.statistics = state.statistics.update(
            state.timeslot,
            block.header.slot,
            block.header.author_index,
            &block.extrinsic,
        );

        // (..., C) Accumulate the available work reports
        //
        // TODO: 12
        crate::accumulate(available)
    };

    // Round 4 computation
    {
        // (δ') Update the accounts
        state.accounts = preimage::accounts(
            block.header.slot,
            &block.extrinsic.preimages,
            &state.accounts,
        )?;

        // (α') Update the authorization pool
        state.pools = guarantee::pools(
            block.header.slot,
            &state.pools,
            &state.authorization,
            &block.extrinsic.guarantees,
        );

        // (β') Update the block history
        let (reported, _) =
            guarantee::report(&state, block.header.slot, &block.extrinsic.guarantees)?;
        state.recent_blocks.import(
            block.header.hash()?,
            block.header.parent_state_root,
            root,
            reported,
        );

        // (τ') Update the timeslot
        state.timeslot = block.header.slot;
    }

    Ok(state)
}

/// (b) Accumulate the available work reports
///
/// TODO: 12
fn accumulate(_available: Vec<WorkReport>) -> OpaqueHash {
    Default::default()
}
