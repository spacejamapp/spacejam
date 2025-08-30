//! Block sync validation

use crate::{
    account::Accounts,
    storage::{Column, Commit},
    Storage,
};
use anyhow::Result;
use pvm::Pvm;
use score::{safrole::ValidatorIter, state::key, Accounts as _, Block, TrieKey};
use serde::Serialize;
use std::sync::Arc;
use tokio::task::JoinSet;

pub mod assurance;
pub mod dispute;
pub mod guarantee;
pub mod preimage;
pub mod ticket;

/// Processor for state transition
pub struct Processor {
    /// The set of encoding tasks
    encode: JoinSet<Result<(TrieKey, Vec<u8>)>>,
}

impl Processor {
    fn new() -> Self {
        Self {
            encode: JoinSet::new(),
        }
    }

    /// Add an encoding task to the pool
    fn encode<T: Serialize + Send + 'static>(&mut self, key: TrieKey, value: T) {
        self.encode.spawn(async move {
            let encoded = codec::encode(&value)?;
            Ok((key, encoded))
        });
    }

    /// Collect all encoding results and populate the diff
    async fn finish(mut self, diff: &mut Commit<TrieKey, Vec<u8>>) -> Result<()> {
        while let Some(result) = self.encode.join_next().await {
            let (key, value) = result??;
            diff.set(key, value);
        }
        Ok(())
    }
}

/// Transit state with new block
#[tracing::instrument(skip_all, name = "stf")]
pub async fn transit<Vm: Pvm>(
    mut block: Block,
    storage: Arc<impl Storage>,
) -> Result<Commit<TrieKey, Vec<u8>>> {
    let diff = self::simulate::<Vm>(&mut block, storage.clone()).await?;
    storage.commit(Column::State, diff.clone())?;
    Ok(diff)
}

/// Simulate state transition with new block
pub async fn simulate<Vm: Pvm>(
    block: &mut Block,
    storage: Arc<impl Storage>,
) -> Result<Commit<TrieKey, Vec<u8>>> {
    let mut state: score::State = storage.state()?;
    let mut diff = Commit::default();
    let mut processor = Processor::new();

    // prepare epoch information
    let epoch = block.header.slot / score::EPOCH_LENGTH;
    let new_epoch: bool = epoch > (state.timeslot / score::EPOCH_LENGTH);
    tracing::debug!("new epoch: {}", new_epoch);

    // handle marks in the block
    {
        // validate the tickets mark
        if let Some(tickets_mark) = block.header.tickets_mark {
            for ticket in tickets_mark {
                if ticket.attempt > score::TICKET_ENTRIES_PER_VALIDATOR as u8 {
                    anyhow::bail!("invalid ticket attempt {}", ticket.attempt);
                }
            }
        }
    }

    // The first round computation
    let mut reports = {
        // (η') Update entropy (6.22)
        tracing::trace!("handle entropy");
        let entropy = crypto::vrf::ietf_output(block.header.entropy_source).unwrap_or_default();
        state.entropy = ticket::eta(new_epoch, &state.entropy, entropy);
        processor.encode(key::ENTROPY, state.entropy);

        // (λ') Update validator state (6.13)
        tracing::trace!("handle previous validators");
        state.validators.previous = state.validators.previous(new_epoch);
        if new_epoch {
            processor.encode(key::PREVIOUS_VALIDATORS, state.validators.previous);
        }

        // (ψ') Update disputes and get marks
        tracing::trace!("handle disputes");
        let (disputes, marks) = self::dispute::disputes(
            state.timeslot,
            &state.validators.current,
            &state.validators.previous,
            &state.disputes,
            &block.extrinsic.disputes,
        )?;
        if disputes != state.disputes {
            processor.encode(key::DISPUTES, disputes.clone());
            state.disputes = disputes;
            block.header.offenders_mark = marks.offenders.clone();
        }

        // (ρ†) Update availability assignments based on verdicts (V) (10.15)
        tracing::trace!("handle availability assignments");
        dispute::reports(&marks, &state.reports)
    };

    // Round 2 computation
    let (available, assurances) = {
        // (W) the sequence of new available work reports (11.16)
        tracing::trace!("handle available work reports");
        let (available, assurances) = self::assurance::available(
            &state.reports,
            &state.validators.current,
            block.header.slot,
            block.header.parent,
            &block.extrinsic.assurances,
        )?;

        // (κ') Update current validators (6.13)
        tracing::trace!("handle current validators");
        state.validators.current = state
            .validators
            .current(new_epoch, &state.safrole.validators);
        if new_epoch {
            processor.encode(key::CURRENT_VALIDATORS, state.validators.current);
        }

        // (ρ‡) Update availability assignments based on assurances (11.17)
        reports = self::assurance::reports(block.header.slot, &available, reports.clone());

        // (ρ') Update availability assignments based on guarantees (11.43)
        reports = guarantee::reports(block.header.slot, &reports, &block.extrinsic.guarantees)?;
        if reports != state.reports {
            processor.encode(key::PENDING_REPORTS, reports.clone());
            state.reports = reports;
        }

        (available, assurances)
    };

    // Round 3 computation
    let (root, accounts) = {
        // (γ') Update the sealing-key series (12.10)
        tracing::trace!("handle sealing-key series");
        state.safrole = ticket::safrole(
            state.timeslot,
            block.header.slot,
            state.entropy,
            &state.disputes.offenders,
            &state.safrole,
            &state.validators,
            &block.extrinsic.tickets,
        )?;
        processor.encode(key::SAFROLE, state.safrole.clone());
        block.header.epoch_mark = state.safrole.epoch_mark(new_epoch, &state.entropy);
        block.header.tickets_mark = state
            .safrole
            .tickets_mark(state.timeslot, block.header.slot);

        // (π') Update the statistic
        tracing::trace!("handle statistic");
        state
            .statistics
            .update(new_epoch, block.header.author_index, &block.extrinsic);
        state.statistics.merge_reports(&available, &assurances);

        // (..., C) Accumulate the available work reports
        tracing::trace!("handle accumulation");
        let accounts = Accounts::new(storage);
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
        )
        .await?;

        state.privileges = accumulation.privileges;
        processor.encode(key::PRIVILEGED_SERVICE, state.privileges.clone());

        state.queue = accumulation.ready_queue;
        processor.encode(key::ACCUMULATION_QUEUE, state.queue.clone());

        state.history = accumulation.accumulated_queue;
        processor.encode(key::ACCUMULATION_HISTORY, state.history.clone());

        state.validators.drawn = accumulation.validators;
        processor.encode(key::DRAWN_VALIDATORS, state.validators.drawn);
        processor.encode(key::ACCUMULATION_LOGS, accumulation.logs);

        state.statistics.merge_services(accumulation.records);
        state.statistics.merge_transfers(accumulation.transfers);
        (accumulation.root, accumulation.accounts)
    };

    // Round 4 computation
    {
        tracing::trace!("handle block history");
        state
            .recent_blocks
            .complete_state_root(block.header.parent_state_root)?;
        let (reported, reporters) = guarantee::report(
            &state,
            block.header.slot,
            &accounts,
            &block.extrinsic.guarantees,
        )?;

        // (β') Update the block history
        state
            .recent_blocks
            .import(block.header.hash()?, root, reported);

        state
            .statistics
            .merge_reporters(&reporters, &state.validators.current.ed25519());
        processor.encode(key::RECENT_BLOCKS, state.recent_blocks.clone());
        processor.encode(key::STATISTICS, state.statistics.clone());

        // (δ') Update the accounts
        tracing::trace!("handle preimages");
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
        tracing::trace!("handle timeslot");
        state.timeslot = block.header.slot;
        processor.encode(key::TIMESLOT, state.timeslot);
    }

    // Finish all encoding tasks in parallel
    processor.finish(&mut diff).await?;

    Ok(diff)
}
