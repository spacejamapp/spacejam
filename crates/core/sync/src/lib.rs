//! Block sync validation

use anyhow::Result;
use score::{
    block::History,
    state::{self, key, Storage},
    validator::Validator,
    work::WorkReport,
    Block, OpaqueHash,
};

pub mod assurance;
pub mod dispute;
pub mod guarantee;
pub mod preimage;
pub mod ticket;

/// Transit state with new block
pub fn transit(block: &Block, validator: impl Validator, storage: impl Storage) -> Result<()> {
    let mut state = storage.state()?;
    let epoch = block.header.slot / score::EPOCH_LENGTH;
    let new_epoch: bool = epoch > (state.timeslot / score::EPOCH_LENGTH);
    let mut diff = vec![];

    // The first round computation
    {
        // (η') Update entropy (6.22)
        let entropy = validator.entropy(state.entropy[0], &block.header.entropy_source)?;
        state.entropy = ticket::eta(new_epoch, &state.entropy, entropy);
        diff.push((key::ENTROPY, codec::encode(&state.entropy)?));

        // (λ') Update validator state (6.13)
        state.validators.previous = state.validators.previous(new_epoch);
        if new_epoch {
            diff.push((
                key::PREVIOUS_VALIDATORS,
                codec::encode(&state.validators.previous)?,
            ));
        }

        // (ψ') Update disputes and get marks
        let (disputes, marks) = crate::dispute::disputes(
            state.timeslot,
            &state.validators.current,
            &state.validators.previous,
            &state.disputes,
            &block.extrinsic.disputes,
        )?;
        if disputes != state.disputes {
            diff.push((key::DISPUTES, codec::encode(&disputes)?));
            state.disputes = disputes;
        }

        // (ρ†) Update availability assignments based on verdicts (V) (10.15)
        let mut reports = dispute::reports(&marks, &state.reports);

        // (ρ‡) Update availability assignments based on assurances (11.17)
        reports = crate::assurance::reports(block.header.slot, reports.clone());

        // (ρ') Update availability assignments based on guarantees (11.43)
        reports = guarantee::reports(
            block.header.slot,
            &state.reports,
            &block.extrinsic.guarantees,
        )?;

        if reports != state.reports {
            diff.push((key::PENDING_REPORTS, codec::encode(&reports)?));
            state.reports = reports;
        }
    }

    // Round 2 computation
    let available = {
        // (κ') Update current validators (6.13)
        state.validators.current = state
            .validators
            .current(new_epoch, &state.safrole.validators);
        if new_epoch {
            diff.push((
                key::CURRENT_VALIDATORS,
                codec::encode(&state.validators.current)?,
            ));
        }

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
        diff.push((key::SAFROLE, codec::encode(&state.safrole)?));

        // (π') Update the statistic
        state.statistics = state.statistics.update(
            state.timeslot,
            block.header.slot,
            block.header.author_index,
            &block.extrinsic,
        );
        diff.push((key::STATISTICS, codec::encode(&state.statistics)?));

        // (..., C) Accumulate the available work reports
        //
        // TODO: 12
        crate::accumulate(available)
    };

    // Round 4 computation
    {
        // (δ') Update the accounts
        let accounts = preimage::accounts(
            block.header.slot,
            &block.extrinsic.preimages,
            &state.accounts,
        )?;
        if accounts != state.accounts {
            diff.extend(state::accounts(&accounts)?);
            state.accounts = accounts;
        }

        // (α') Update the authorization pool
        let pools = guarantee::pools(
            block.header.slot,
            &state.pools,
            &state.authorization,
            &block.extrinsic.guarantees,
        );
        if pools != state.pools {
            diff.push((key::AUTHORIZATION_POOLS, codec::encode(&pools)?));
            state.pools = pools;
        }

        // (β') Update the block history
        let (reported, _) =
            guarantee::report(&state, block.header.slot, &block.extrinsic.guarantees)?;
        state.recent_blocks.import(
            block.header.hash()?,
            block.header.parent_state_root,
            root,
            reported,
        );
        diff.push((key::RECENT_BLOCKS, codec::encode(&state.recent_blocks)?));

        // (τ') Update the timeslot
        state.timeslot = block.header.slot;
        diff.push((key::TIMESLOT, codec::encode(&state.timeslot)?));
    }

    storage.stash(block.header.hash()?, diff)?;
    Ok(())
}

/// (b) Accumulate the available work reports
///
/// TODO: 12
fn accumulate(_available: Vec<WorkReport>) -> OpaqueHash {
    Default::default()
}
