//! Reporting is the process of reporting the results of a work-package to the service state singleton.

use crate::{
    extrinsic::GuaranteesExtrinsic,
    runtime::{
        vm::{StateContext, Vm},
        Storage,
    },
    service::{
        AccumulatedQueue, AvailabilityAssignment, AvailabilityAssignments, Privileges, ReadyQueue,
        ReportedWorkPackage, WorkReport,
    },
    Ed25519Public, OpaqueHash, TimeSlot, CORES_COUNT,
};
use dep::Dependencies;
use error::{Error, Result};
pub use {
    exec::ExecResult,
    state::{State, StateJson},
};

mod dep;
pub mod error;
mod exec;
mod queue;
mod state;
mod validator;

/// (b) Accumulate the available work reports
pub fn accumulate<V: Vm>(
    // The next timeslot (τ')
    slot: TimeSlot,
    // The prior timeslot (τ)
    _tau: TimeSlot,
    // available work reports (W)
    reports: Vec<WorkReport>,
    // The ready queue (θ)
    ready_queue: &mut ReadyQueue,
    // The accumulated queue (ξ)
    accumulated_queue: &mut AccumulatedQueue,
    // The privileges (χ)
    privileges: &Privileges,
    // The account storage (δ)
    accounts: &impl Storage,
) -> anyhow::Result<OpaqueHash> {
    // (W*) get accumulatable work reports
    let accumulatable = queue::accumulatable(slot, reports, ready_queue, accumulated_queue);

    // (Δ+) run outer accumulation
    let gas_limit = privileges.gas_limit();
    let _result = exec::exec::<V>(
        gas_limit,
        accumulatable,
        StateContext::default(),
        accounts,
        &privileges.always_acc,
    );

    Ok(Default::default())
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
    pools: &[Vec<OpaqueHash>; crate::CORES_COUNT],
    authorizations: &[Vec<OpaqueHash>; crate::CORES_COUNT],
    guarantees: &GuaranteesExtrinsic,
) -> [Vec<OpaqueHash>; crate::CORES_COUNT] {
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
    state: &crate::State,
    slot: TimeSlot,
    guarantees: &GuaranteesExtrinsic,
) -> Result<(Vec<ReportedWorkPackage>, Vec<Ed25519Public>)> {
    let pstate = State::from(state.clone());
    let mut validator = validator::GuaranteeValidator::from(&pstate);
    validator.validate(slot, guarantees)
}
