//! Reporting is the process of reporting the results of a work-package to the service state singleton.

pub use acc::Accumulation;
use account::Accounts;
use error::{Error, Result};
use pvm::{AccumulateState, Pvm};
use score::{
    AUTH_QUEUE_SIZE, Array, CORES_COUNT, Ed25519Public, EntropyBuffer, OpaqueHash, TimeSlot,
    extrinsic::GuaranteesExtrinsic,
    safrole::ValidatorsData,
    service::{
        AccumulatedQueue, AvailabilityAssignment, AvailabilityAssignments, Privileges, ReadyQueue,
        ReadyReport, ReportedWorkPackage, WorkReport,
    },
};

mod acc;
pub mod error;
mod exec;
mod queue;
mod validator;

/// (b) Accumulate the available work reports
#[tracing::instrument(skip_all)]
#[allow(clippy::too_many_arguments)]
pub fn accumulate<V: Pvm, R: Accounts>(
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
    // The validators to be drawn (ι)
    validators: &ValidatorsData,
    // The authorization queue (φ)
    authorization: &Array<Array<OpaqueHash, AUTH_QUEUE_SIZE>, CORES_COUNT>,
    // The account storage (δ)
    accounts: R,
    // The entropy (η)
    entropy: EntropyBuffer,
) -> anyhow::Result<Accumulation<R>> {
    // (W*) get accumulatable work reports
    let (accumulatable, queued) =
        queue::accumulatable(slot, reports, ready_queue, accumulated_queue);

    // Seed φ from prior state so the assign host call mutates the real queue
    let auth_init: score::AuthorizationPools = authorization
        .iter()
        .map(|q| q.to_vec())
        .collect::<Vec<_>>()
        .try_into()
        .expect("authorization has CORES_COUNT entries");

    // (Δ+) run outer accumulation (12.18)
    let gas_limit = privileges.gas_limit();
    let mut validators = validators.clone();
    let mut accumulated = exec::outer::<V, R>(
        gas_limit,
        Default::default(),
        &accumulatable,
        AccumulateState {
            accounts,
            privileges: privileges.clone(),
            authorization: auth_init,
            entropy,
            timeslot: slot,
        },
        &mut validators,
        &privileges.always_acc,
    )?;

    // (πS') compose the service activity records
    let n = accumulated.accumulated;
    let records = accumulated.records(&accumulatable[..n]);

    // update the accumulated queue (ξ')
    let next_accumulated_queue = self::accumulated_history(accumulated_queue, accumulatable, n);

    // update the ready queue (θ')
    let next_ready_queue =
        self::ready_queue(ready_queue, &next_accumulated_queue, queued, tau, slot);

    // (φ') Project the per-core Vec back to the fixed-size state shape
    let next_authorization: Array<Array<OpaqueHash, AUTH_QUEUE_SIZE>, CORES_COUNT> =
        std::mem::take(&mut accumulated.context.authorization)
            .into_iter()
            .map(|q| Array::try_from(q).expect("queue has AUTH_QUEUE_SIZE entries"))
            .collect::<Vec<_>>()
            .try_into()
            .expect("authorization has CORES_COUNT entries");

    Ok(Accumulation {
        root: accumulated.root(),
        ready_queue: next_ready_queue,
        accumulated_queue: next_accumulated_queue,
        accounts: accumulated.context.accounts,
        privileges: accumulated.context.privileges,
        validators,
        records,
        authorization: next_authorization,
        logs: accumulated.pairings,
    })
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

    // Set of newly-accumulated work-package hashes, canonically sorted.
    let mut new_accumulated: Vec<OpaqueHash> = accumulatable
        .iter()
        .take(accumulated)
        .map(|w| w.spec.hash)
        .collect();
    new_accumulated.sort();
    next.push(new_accumulated);

    // Update the accumulated history (ξ')
    let mut history = AccumulatedQueue::default();
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
    tau: TimeSlot,
    slot: TimeSlot,
) -> ReadyQueue {
    let mut ready_queue = pre.clone();
    let phase = slot % score::EPOCH_LENGTH;
    let accd = history[score::EPOCH_LENGTH as usize - 1].clone();

    // update the ready queue (θ')
    //
    // saturating_sub: header validation rejects slot <= tau, but it runs in
    // parallel with STF simulation in block::process — must not panic on the
    // simulate side before the joined match bails on the validate error.
    let blocks = slot.saturating_sub(tau);
    for idx in 0..score::EPOCH_LENGTH {
        let target = ((score::EPOCH_LENGTH + phase - idx) % score::EPOCH_LENGTH) as usize;
        let ready = if idx == 0 {
            queue::edit(reports.clone(), &accd)
        } else if idx >= 1 && idx < blocks {
            Default::default()
        } else if idx >= blocks {
            queue::edit(pre[target].clone(), &accd)
        } else {
            continue;
        };

        ready_queue[target] = ready;
    }

    ready_queue
}

/// (ρ') Update availability assignments based on guarantees (11.43)
pub fn reports(
    slot: TimeSlot,
    prev: &AvailabilityAssignments,
    guarantees: &GuaranteesExtrinsic,
) -> Result<AvailabilityAssignments> {
    let mut next = prev.clone();
    let mut lcoreidx = 0;
    for guarantee in guarantees.iter() {
        let core_index = guarantee.report.core_index as usize;
        if core_index < lcoreidx {
            return Err(Error::OutOfOrderGuarantee);
        }

        lcoreidx = core_index;
        if core_index >= CORES_COUNT {
            return Err(Error::BadCoreIndex);
        }

        if matches!(prev.get(core_index), Some(Some(_))) {
            return Err(Error::CoreEngaged);
        }

        if guarantee.report.results.is_empty() {
            return Err(Error::MissingWorkResults);
        }

        next[core_index] = Some(AvailabilityAssignment {
            report: guarantee.report.clone(),
            timeout: slot,
        });
    }

    Ok(next)
}

/// (α') Update authorization pools.
pub fn pools(
    timeslot: TimeSlot,
    pools: &score::AuthorizationPools,
    authorizations: &score::Array<
        score::Array<OpaqueHash, { score::AUTH_QUEUE_SIZE }>,
        { score::CORES_COUNT },
    >,
    guarantees: &GuaranteesExtrinsic,
) -> score::AuthorizationPools {
    let slot = timeslot as usize % score::AUTH_QUEUE_SIZE;
    let mut new_pools = score::AuthorizationPools::default();
    for (core_index, pool) in pools.iter().enumerate() {
        let mut new_pool = pool.clone();

        // remove old authorizers from the pool
        for guarantee in guarantees {
            if guarantee.report.core_index as usize == core_index
                && let Some(pos) = new_pool
                    .iter()
                    .position(|auth| *auth == guarantee.report.authorizer_hash)
            {
                new_pool.remove(pos);
            }
        }

        // add new authorizer from queue at position H_t mod Q
        new_pool.push(authorizations[core_index][slot]);

        // truncate the pool to the max size
        if let Some(old) = new_pool.len().checked_sub(score::AUTH_POOL_MAX_SIZE) {
            new_pool = new_pool[old..].into();
        }

        new_pools[core_index] = new_pool;
    }

    new_pools
}

/// (p of β') Report the work packages
pub fn report(
    state: &score::State,
    slot: TimeSlot,
    services: &impl Accounts,
    guarantees: &GuaranteesExtrinsic,
) -> Result<(Vec<ReportedWorkPackage>, Vec<Ed25519Public>)> {
    let mut validator = validator::GuaranteeValidator::new(state, services);
    validator.validate(slot, guarantees)
}
