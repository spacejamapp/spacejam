//! Block sync validation

use crate::{
    Storage,
    account::Accounts,
    storage::{Column, Commit},
    timing,
};
use account::Accounts as _;
use anyhow::Result;
use pvm::Pvm;
use score::{Block, TrieKey, safrole::ValidatorIter};
use std::{sync::Arc, thread};

pub mod assurance;
pub mod block;
pub mod dispute;
pub mod guarantee;
pub mod preimage;
pub mod ticket;

/// Transit state with new block
#[tracing::instrument(skip_all, name = "stf")]
pub fn transit<Vm: Pvm>(
    mut block: Block,
    storage: Arc<impl Storage>,
) -> Result<Commit<TrieKey, Vec<u8>>> {
    let diff = self::simulate::<Vm>(&mut block, storage.clone())?;
    let _guard = timing::commit();
    storage.commit(Column::State, diff.clone())?;
    Ok(diff)
}

/// Transit state with new block
#[tracing::instrument(skip_all, name = "stf")]
pub fn transit_with_state<Vm: Pvm>(
    mut block: Block,
    state: score::State,
    storage: Arc<impl Storage>,
) -> Result<Commit<TrieKey, Vec<u8>>> {
    let diff = self::simulate_with_state::<Vm>(&mut block, state, storage.clone())?;
    let _guard = timing::commit();
    storage.commit(Column::State, diff.clone())?;
    Ok(diff)
}

/// Simulate state transition with new block
pub fn simulate<Vm: Pvm>(
    block: &mut Block,
    storage: Arc<impl Storage>,
) -> Result<Commit<TrieKey, Vec<u8>>> {
    let state = storage.state()?;
    self::simulate_with_state::<Vm>(block, state, storage.clone())
}

/// Simulate state transition with new block
pub fn simulate_with_state<Vm: Pvm>(
    block: &mut Block,
    mut state: score::State,
    storage: Arc<impl Storage>,
) -> Result<Commit<TrieKey, Vec<u8>>> {
    let mut diff = Commit::default();

    // prepare epoch information
    let epoch = block.header.slot / score::EPOCH_LENGTH;
    let new_epoch: bool = epoch > (state.timeslot / score::EPOCH_LENGTH);
    state.check(&block.header, new_epoch)?;

    // The first round computation
    let accounts = Accounts::new(storage);
    let (mut reports, reported, reporters) = {
        // (η') Update entropy (6.22)
        {
            let _guard = timing::entropy();
            let entropy = crypto::vrf::ietf_output(block.header.entropy_source).unwrap_or_default();
            state.entropy = ticket::eta(new_epoch, &state.entropy, entropy);
        };

        // (λ') Update validator state (6.13)
        if new_epoch {
            state.validators.previous = state.validators.previous(new_epoch);
        }

        // (ψ') Update disputes and get marks
        let marks = if block.extrinsic.disputes.is_empty() {
            Default::default()
        } else {
            let _guard = timing::disputes();
            let (disputes, marks) = self::dispute::disputes(
                state.timeslot,
                &state.validators.current,
                &state.validators.previous,
                &state.disputes,
                &block.extrinsic.disputes,
            )?;

            state.disputes = disputes;
            {
                if block.header.offenders_mark != marks.offenders {
                    anyhow::bail!("offenders mark mismatch");
                }
                // FIXME: for building blocks only, could be removed
                // on importing blocks.
                // block.header.offenders_mark = marks.offenders.clone();
            }
            marks
        };

        // (p of β') validate the guarantees
        let (mut reported, mut reporters) = (vec![], vec![]);
        if !block.extrinsic.guarantees.is_empty() {
            (reported, reporters) = {
                let _guard = timing::guarantees();
                guarantee::report(
                    &state,
                    block.header.slot,
                    &accounts,
                    &block.extrinsic.guarantees,
                )?
            }
        };

        // (ρ†) Update availability assignments based on verdicts (V) (10.15)
        let _guard = timing::assignments();
        (
            dispute::reports(&marks, &state.reports),
            reported,
            reporters,
        )
    };

    // Round 2 computation
    let (available, assurances) = {
        // (W) the sequence of new available work reports (11.16)
        let (available, assurances) = {
            let _guard = timing::assurances();
            self::assurance::available(
                &state.reports,
                &state.validators.current,
                block.header.slot,
                block.header.parent,
                &block.extrinsic.assurances,
            )?
        };

        // (κ') Update current validators (6.13)
        if new_epoch {
            state.validators.current = state
                .validators
                .current(new_epoch, &state.safrole.validators);
        }

        // (ρ‡) Update availability assignments based on assurances (11.17)
        reports = self::assurance::reports(block.header.slot, &available, reports.clone());

        // (ρ') Update availability assignments based on guarantees (11.43)
        state.reports =
            guarantee::reports(block.header.slot, &reports, &block.extrinsic.guarantees)?;
        (available, assurances)
    };

    // Round 3 computation
    let (root, accounts) = {
        // (γ') Update the sealing-key series (12.10)
        if !block.extrinsic.tickets.is_empty() || new_epoch {
            let _guard = timing::safrole();
            state.safrole = ticket::safrole(
                state.timeslot,
                block.header.slot,
                state.entropy,
                &state.disputes.offenders,
                &state.safrole,
                &state.validators,
                &block.extrinsic.tickets,
            )?;

            {
                // FIXME: for building blocks only, could be removed
                // on importing blocks.
                if new_epoch {
                    block.header.epoch_mark = state.safrole.epoch_mark(&state.entropy);
                }
                block.header.tickets_mark = state
                    .safrole
                    .tickets_mark(state.timeslot, block.header.slot);
            }
        }

        // (π') Update the statistic
        state
            .statistics
            .update(new_epoch, block.header.author_index, &block.extrinsic);
        state.statistics.merge_reports(&available, &assurances);

        // (..., C) Accumulate the available work reports
        let _guard = timing::accumulate();
        let accumulation = guarantee::accumulate::<Vm, _>(
            block.header.slot,
            state.timeslot,
            available,
            &state.queue,
            &state.history,
            &state.privileges,
            &state.validators.drawn,
            accounts,
            state.entropy,
        )?;

        // lazy load vrf rings
        if state.validators.drawn != accumulation.validators {
            thread::spawn(move || {
                ticket::lazy::drawn(
                    if new_epoch { epoch + 1 } else { epoch + 2 },
                    &accumulation.validators,
                )
            });
        }

        // update state fields
        state.privileges = accumulation.privileges;
        state.queue = accumulation.ready_queue;
        state.history = accumulation.accumulated_queue;
        state.validators.drawn = accumulation.validators;
        state.statistics.merge_services(accumulation.records);
        state.logs = accumulation.logs;
        (accumulation.root, accumulation.accounts)
    };

    // Round 4 computation
    {
        // (β') Update the block history
        block::history::import(
            &mut state.recent_blocks,
            block.header.hash(),
            root,
            reported,
        );

        if !reporters.is_empty() {
            state
                .statistics
                .merge_reporters(&reporters, &state.validators.current.ed25519());
        }

        // (δ') Update the accounts
        // if !block.extrinsic.preimages.is_empty() {
        // let _guard = timing::preimages();
        let accounts = preimage::accounts(block.header.slot, &block.extrinsic.preimages, accounts)?;
        let (updates, removals) = accounts.diff();
        diff.extend_iter(updates, removals);
        // }

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
    }

    diff.update.extend(state.pairs(new_epoch, &block.extrinsic));
    Ok(diff)
}
