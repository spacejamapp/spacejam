//! Block sync validation

use crate::{Config, Storage, account::Accounts, storage::Commit};
use anyhow::Result;
use pvm::Pvm;
use score::{Block, StorageKey, account::Accounts as _, block::History, state::key};
use std::sync::Arc;

pub mod assurance;
pub mod dispute;
pub mod guarantee;
pub mod preimage;
pub mod ticket;

/// Transit state with new block
#[tracing::instrument(skip_all, name = "stf")]
pub fn transit<Vm: Pvm>(
    mut block: Block,
    storage: Arc<impl Storage>,
) -> Result<Commit<StorageKey, Vec<u8>>> {
    let diff = self::simulate::<Vm>(&mut block, storage.clone())?;
    storage.commit(diff.clone())?;
    Ok(diff)
}

/// Simulate state transition with new block
pub fn simulate<Vm: Pvm>(
    block: &mut Block,
    storage: Arc<impl Storage>,
) -> Result<Commit<StorageKey, Vec<u8>>> {
    let mut state: score::State = storage.state()?;
    let mut diff = Commit::default();

    // prepare epoch information
    let epoch = block.header.slot / score::EPOCH_LENGTH;
    let new_epoch: bool = epoch > (state.timeslot / score::EPOCH_LENGTH);

    // The first round computation
    let mut reports = {
        // (η') Update entropy (6.22)
        let entropy = crypto::vrf::ietf_output(block.header.entropy_source).unwrap_or_default();
        state.entropy = ticket::eta(new_epoch, &state.entropy, entropy);
        diff.set(key::ENTROPY, codec::encode(&state.entropy)?);

        // (λ') Update validator state (6.13)
        state.validators.previous = state.validators.previous(new_epoch);
        if new_epoch {
            diff.set(
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
            diff.set(key::DISPUTES, codec::encode(&disputes)?);
            state.disputes = disputes;
            block.header.offenders_mark = marks.offenders.clone();
        }

        // (ρ†) Update availability assignments based on verdicts (V) (10.15)
        dispute::reports(&marks, &state.reports)
    };

    // Round 2 computation
    let (available, assurances) = {
        // (κ') Update current validators (6.13)
        state.validators.current = state
            .validators
            .current(new_epoch, &state.safrole.validators);
        if new_epoch {
            diff.set(
                key::CURRENT_VALIDATORS,
                codec::encode(&state.validators.current)?,
            );
        }

        // (W) the sequence of new available work reports (11.16)
        let (available, assurances) = self::assurance::available(
            &state.reports,
            &state.validators.current,
            block.header.slot,
            block.header.parent,
            &block.extrinsic.assurances,
        )?;

        // (ρ‡) Update availability assignments based on assurances (11.17)
        reports = self::assurance::reports(block.header.slot, &available, reports.clone());

        // (ρ') Update availability assignments based on guarantees (11.43)
        reports = guarantee::reports(block.header.slot, &reports, &block.extrinsic.guarantees)?;
        if reports != state.reports {
            diff.set(key::PENDING_REPORTS, codec::encode(&reports)?);
            state.reports = reports;
        }

        (available, assurances)
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
        diff.set(key::SAFROLE, codec::encode(&state.safrole)?);
        block.header.epoch_mark = state.safrole.epoch_mark(new_epoch, &state.entropy);
        block.header.tickets_mark = state
            .safrole
            .tickets_mark(state.timeslot, block.header.slot);

        // (π') Update the statistic
        state.statistics.update(
            block.header.slot,
            block.header.author_index,
            &block.extrinsic,
        );
        state.statistics.merge_reports(&available, &assurances);

        // (..., C) Accumulate the available work reports
        let accounts = Accounts::new(storage);
        let accumulation = guarantee::accumulate::<Vm, _>(
            block.header.slot,
            state.timeslot,
            available,
            &state.queue,
            &state.history,
            &state.privileges,
            accounts,
        )?;

        state.privileges = accumulation.privileges;
        diff.set(key::PRIVILEGED_SERVICE, codec::encode(&state.privileges)?);

        state.queue = accumulation.ready_queue;
        diff.set(key::ACCUMULATION_QUEUE, codec::encode(&state.queue)?);

        state.history = accumulation.accumulated_queue;
        diff.set(key::ACCUMULATION_HISTORY, codec::encode(&state.history)?);

        // write statistics and return root and accounts
        state.statistics.merge_services(accumulation.records);
        diff.set(key::STATISTICS, codec::encode(&state.statistics)?);
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

        diff.set(key::RECENT_BLOCKS, codec::encode(&state.recent_blocks)?);

        // (δ') Update the accounts
        let accounts = preimage::accounts(block.header.slot, &block.extrinsic.preimages, accounts)?;
        let (updates, removals) = accounts.diff();
        diff.extend_iter(updates, removals);

        // FIXME: looks like polkajam currently doesn't update the authorization
        // pool, so we're not updating it here as well atm.
        //
        // // (α') Update the authorization pool
        // let pools = guarantee::pools(
        //     block.header.slot,
        //     &state.pools,
        //     &state.authorization,
        //     &block.extrinsic.guarantees,
        // );
        // if pools != state.pools {
        //     diff.insert(key::AUTHORIZATION_POOLS, codec::encode(&pools)?);
        //     state.pools = pools;
        // }

        // (τ') Update the timeslot
        state.timeslot = block.header.slot;
        diff.set(key::TIMESLOT, codec::encode(&state.timeslot)?);
    }

    Ok(diff)
}
