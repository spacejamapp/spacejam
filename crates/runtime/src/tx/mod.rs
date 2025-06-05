//! Block sync validation

use crate::Storage;
use anyhow::Result;
use pvm::Pvm;
use score::{
    Block, StorageKey,
    block::History,
    state::{account, key},
};
use std::collections::HashMap;

pub mod assurance;
pub mod dispute;
pub mod guarantee;
pub mod preimage;
pub mod ticket;

/// Transit state with new block
#[tracing::instrument(skip_all, name = "stf")]
pub fn transit<V: Pvm>(
    mut block: Block,
    storage: &impl Storage,
) -> Result<HashMap<StorageKey, Vec<u8>>> {
    let diff = self::simulate::<V>(&mut block, storage)?;
    storage.commit(diff.iter().map(|(k, v)| (k.to_vec(), v.clone())).collect())?;
    Ok(diff)
}

/// Simulate state transition with new block
pub fn simulate<V: Pvm>(
    block: &mut Block,
    storage: &impl Storage,
) -> Result<HashMap<StorageKey, Vec<u8>>> {
    let mut state: score::State = storage.state()?;
    let mut diff = HashMap::new();

    // prepare epoch information
    let epoch = block.header.slot / score::EPOCH_LENGTH;
    let new_epoch: bool = epoch > (state.timeslot / score::EPOCH_LENGTH);

    // The first round computation
    {
        // (η') Update entropy (6.22)
        let entropy = crypto::vrf::ietf_output(block.header.entropy_source).unwrap_or_default();
        state.entropy = ticket::eta(new_epoch, &state.entropy, entropy);
        diff.insert(key::ENTROPY, codec::encode(&state.entropy)?);

        // (λ') Update validator state (6.13)
        state.validators.previous = state.validators.previous(new_epoch);
        if new_epoch {
            diff.insert(
                key::PREVIOUS_VALIDATORS,
                codec::encode(&state.validators.previous)?,
            );
        }

        // (ψ') Update disputes and get marks
        let (disputes, marks) = self::dispute::disputes(
            state.timeslot,
            &state.validators.current,
            &state.validators.previous,
            &state.disputes,
            &block.extrinsic.disputes,
        )?;
        if disputes != state.disputes {
            diff.insert(key::DISPUTES, codec::encode(&disputes)?);
            state.disputes = disputes;
            block.header.offenders_mark = marks.offenders.clone();
        }

        // (ρ†) Update availability assignments based on verdicts (V) (10.15)
        let mut reports = dispute::reports(&marks, &state.reports);

        // (ρ‡) Update availability assignments based on assurances (11.17)
        reports = self::assurance::reports(block.header.slot, reports.clone());

        // (ρ') Update availability assignments based on guarantees (11.43)
        reports = guarantee::reports(block.header.slot, &reports, &block.extrinsic.guarantees)?;
        if reports != state.reports {
            diff.insert(key::PENDING_REPORTS, codec::encode(&reports)?);
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
            diff.insert(
                key::CURRENT_VALIDATORS,
                codec::encode(&state.validators.current)?,
            );
        }

        // (W*) the sequence of new available work reports (11.16)
        self::assurance::available(
            &state.reports,
            &state.validators.current,
            block.header.slot,
            block.header.parent,
            &block.extrinsic.assurances,
        )?
    };

    // Round 3 computation
    let (root, accounts) = {
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
        diff.insert(key::SAFROLE, codec::encode(&state.safrole)?);
        block.header.epoch_mark = state.safrole.epoch_mark(new_epoch, &state.entropy);
        block.header.tickets_mark = state
            .safrole
            .tickets_mark(state.timeslot, block.header.slot);

        // (π') Update the statistic
        state.statistics = state.statistics.update(
            block.header.slot,
            block.header.author_index,
            &block.extrinsic,
        );
        diff.insert(key::STATISTICS, codec::encode(&state.statistics)?);

        // (..., C) Accumulate the available work reports
        let accumulation = guarantee::accumulate::<V>(
            block.header.slot,
            state.timeslot,
            available,
            &state.queue,
            &state.history,
            &state.privileges,
            state.accounts.clone(),
        )?;
        state.queue = accumulation.ready_queue;
        state.history = accumulation.accumulated_queue;
        state.privileges = accumulation.privileges;
        (accumulation.root, accumulation.accounts)
    };

    // Round 4 computation
    {
        // (β') Update the block history
        state.recent_blocks.import(
            block.header.hash()?,
            block.header.parent_state_root,
            root,
            Default::default(),
        );
        let (reported, _) =
            guarantee::report(&state, block.header.slot, &block.extrinsic.guarantees)?;
        if let Some(last) = state.recent_blocks.last_mut() {
            last.reported = reported;
        };

        diff.insert(key::RECENT_BLOCKS, codec::encode(&state.recent_blocks)?);

        // (δ') Update the accounts
        let accounts =
            preimage::accounts(block.header.slot, &block.extrinsic.preimages, &accounts)?;
        if accounts != state.accounts {
            diff.extend(account::diff(&accounts)?);
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
            tracing::info!(
                "updating pools: {:?}",
                pools
                    .iter()
                    .map(|c| c.iter().map(|v| hex::encode(v)).collect::<Vec<_>>())
                    .collect::<Vec<_>>()
            );
            diff.insert(key::AUTHORIZATION_POOLS, codec::encode(&pools)?);
            state.pools = pools;
        }

        // (τ') Update the timeslot
        state.timeslot = block.header.slot;
        diff.insert(key::TIMESLOT, codec::encode(&state.timeslot)?);
    }

    Ok(diff)
}
