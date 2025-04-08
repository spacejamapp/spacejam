//! Reporting is the process of reporting the results of a work-package to the service state singleton.

use error::{Error, Result};
use pvm::Pvm;
use score::{
    CORES_COUNT, Ed25519Public, OpaqueHash, TimeSlot,
    extrinsic::GuaranteesExtrinsic,
    service::{
        AccumulatedQueue, AvailabilityAssignment, AvailabilityAssignments, Privileges, ReadyQueue,
        ReadyReport, ReportedWorkPackage, ServiceAccount, WorkReport,
    },
    vm::StateContext,
};
pub use state::{State, StateJson};
use std::collections::BTreeMap;

mod dep;
pub mod error;
mod exec;
mod queue;
mod state;
mod validator;

/// (b) Accumulate the available work reports
pub fn accumulate<V: Pvm>(
    // The next timeslot (τ')
    slot: TimeSlot,
    // The prior timeslot (τ)
    tau: TimeSlot,
    // available work reports (W)
    reports: Vec<WorkReport>,
    // The ready queue (θ)
    ready_queue: &ReadyQueue,
    // The accumulated queue (ξ)
    accumulated_queue: &AccumulatedQueue,
    // The privileges (χ)
    privileges: &Privileges,
    // The account storage (δ)
    accounts: BTreeMap<u32, ServiceAccount>,
) -> anyhow::Result<(OpaqueHash, ReadyQueue, AccumulatedQueue)> {
    // (W*) get accumulatable work reports
    let (accumulatable, queued) =
        queue::accumulatable(slot, reports, ready_queue, accumulated_queue);

    // (Δ+) run outer accumulation
    let gas_limit = privileges.gas_limit();
    let accumulated = exec::outer::<V>(
        gas_limit,
        &accumulatable,
        StateContext {
            accounts,
            ..Default::default()
        },
        &privileges.always_acc,
    );

    // update the accumulated queue (ξ')
    let next_accumulated_queue =
        self::accumulated_history(accumulated_queue, accumulatable, accumulated.accumulated);

    // update the ready queue (θ')
    let next_ready_queue =
        self::ready_queue(ready_queue, &next_accumulated_queue, queued, slot, tau);

    // TODO: note that we need to update account data as well after accumulation

    Ok((Default::default(), next_ready_queue, next_accumulated_queue))
}

/// (ξ') Update the accumulated history
pub fn accumulated_history(
    pre: &AccumulatedQueue,
    accumulatable: Vec<WorkReport>,
    accumulated: usize,
) -> AccumulatedQueue {
    let mut next = pre.to_vec();
    // Update accumulated history (keeping last E entries where E is epoch length)
    if next.len() >= score::EPOCH_LENGTH as usize {
        next.remove(0);
    }

    // Add new accumulated work report hashes
    let new_accumulated: Vec<OpaqueHash> = accumulatable
        .iter()
        .take(accumulated)
        .map(|w| w.spec.hash)
        .collect();

    next.push(new_accumulated);

    // Update the accumulated history (ξ')
    let mut history: [Vec<OpaqueHash>; score::EPOCH_LENGTH as usize] = Default::default();
    for (i, item) in next.iter().enumerate() {
        history[i] = item.clone();
    }
    history
}

/// (θ') Update the ready queue
pub fn ready_queue(
    pre: &ReadyQueue,
    history: &AccumulatedQueue,
    reports: Vec<ReadyReport>,
    slot: TimeSlot,
    tau: TimeSlot,
) -> ReadyQueue {
    let mut ready_queue = pre.clone();
    let slot_idx = slot / score::EPOCH_LENGTH;
    let accd = history[score::EPOCH_LENGTH as usize - 1].clone();

    // update the ready queue (θ')
    let blocks = slot - tau;
    for idx in 0..slot_idx {
        let target = slot_idx - idx;
        let ready = if idx == 0 {
            queue::edit(reports.clone(), &accd)
        } else if idx >= 1 && idx < blocks {
            Default::default()
        } else if idx >= blocks {
            queue::edit(pre[target as usize].clone(), &accd)
        } else {
            continue;
        };

        ready_queue[target as usize] = ready;
    }

    ready_queue
}

/// (ρ') Update availability assignments based on guarantees
pub fn reports(
    slot: TimeSlot,
    prev: &AvailabilityAssignments,
    guarantees: &GuaranteesExtrinsic,
) -> Result<AvailabilityAssignments> {
    let mut next = prev.clone();
    for guarantee in guarantees.iter() {
        let core_index = guarantee.report.core_index as usize;
        if core_index >= CORES_COUNT {
            return Err(Error::BadCoreIndex);
        }

        if let Some(Some(assignment)) = prev.get(core_index) {
            if slot <= assignment.timeout + 1 {
                return Err(Error::CoreEngaged);
            }
        }

        next[core_index] = Some(AvailabilityAssignment {
            report: guarantee.report.clone(),
            timeout: slot,
        });
    }

    Ok(next)
}

/// (α') Update authorization pools.
///
/// TODO: check indices
pub fn pools(
    slot: TimeSlot,
    pools: &[Vec<OpaqueHash>; score::CORES_COUNT],
    authorizations: &[Vec<OpaqueHash>; score::CORES_COUNT],
    guarantees: &GuaranteesExtrinsic,
) -> [Vec<OpaqueHash>; score::CORES_COUNT] {
    let mut pools = pools.clone();

    // Process each guarantee
    let mut processed = Vec::new();
    for guarantee in guarantees {
        // Consume the authorizer from the pool
        pools[guarantee.report.core_index as usize] = pools[guarantee.report.core_index as usize]
            .iter()
            .filter(|pool| **pool != guarantee.report.authorizer_hash)
            .cloned()
            .collect();

        // mark the core as processed
        processed.push(guarantee.report.core_index as usize);
    }

    // add new authorizers from queue to the pools
    for (core_index, pool) in pools.iter_mut().enumerate() {
        if !processed.contains(&core_index) && !pool.is_empty() {
            *pool = pool[1..].into();
        }

        // TODO: recheck this logic
        if let Some(auth) = authorizations[core_index].get(slot as usize) {
            pool.push(*auth);
        }
    }

    pools
}

/// Report the work packages
///
/// TODO: refactor the state on connecting storage.
pub fn report(
    state: &score::State,
    slot: TimeSlot,
    guarantees: &GuaranteesExtrinsic,
) -> Result<(Vec<ReportedWorkPackage>, Vec<Ed25519Public>)> {
    let pstate = State::from(state.clone());
    let mut validator = validator::GuaranteeValidator::from(&pstate);
    validator.validate(slot, guarantees)
}
