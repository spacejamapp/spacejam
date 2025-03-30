//! Accumulation of guarantees

use crate::{
    runtime::Storage,
    service::{AccumulatedQueue, Privileges, ReadyQueue, WorkReport},
    OpaqueHash, TimeSlot,
};

/// (b) Accumulate the available work reports
pub fn accumulate(
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
    _privileges: &Privileges,
    // The account storage (δ)
    _accounts: &impl Storage,
) -> anyhow::Result<OpaqueHash> {
    let _accumulatable = self::accumulatable(slot, reports, ready_queue, accumulated_queue);

    Ok(Default::default())
}

/// Extracts the accumulatable work reports
///
/// ref GP: 12.1 Hisotry and Queuing
fn accumulatable(
    slot: TimeSlot,
    reports: Vec<WorkReport>,
    ready_queue: &ReadyQueue,
    accumulated_queue: &AccumulatedQueue,
) -> Vec<WorkReport> {
    let accd = accumulated_queue
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();

    // (W_!) work reports to be accumulated immediately
    let (mut acci, mut accq) = (vec![], vec![]);
    for report in reports {
        if report.is_immediate() {
            acci.push(report);
        } else {
            accq.push(report);
        }
    }

    // (W_Q) work reports to be queued for accumulation
    let accq: Vec<(WorkReport, Vec<OpaqueHash>)> = self::pairing(accq, &accd);

    // construct the priority queue
    let mid = (slot % crate::EPOCH_LENGTH) as usize;
    let ready = ready_queue
        .iter()
        .flat_map(|r| r.iter().map(|r| (r.report.clone(), r.dependencies.clone())))
        .collect::<Vec<_>>();

    // extract the work package hashes
    self::priority(self::edit(
        [ready[mid..].to_vec(), ready[..mid].to_vec(), accq].concat(),
        &self::mapping(&acci),
    ))
}

/// (D) pairing work reports with their dependencies
fn pairing(accq: Vec<WorkReport>, accd: &[OpaqueHash]) -> Vec<(WorkReport, Vec<OpaqueHash>)> {
    self::edit(
        accq.into_iter()
            .map(|report| {
                let deps = report
                    .context
                    .prerequisites
                    .iter()
                    .cloned()
                    .chain(report.lookup.iter().map(|lookup| lookup.hash))
                    .collect::<Vec<_>>();

                (report, deps)
            })
            .collect(),
        accd,
    )
}

/// (E) queue-editing function
///
/// Removes the accumulated dependencies from the accumulated queue
fn edit(
    accq: Vec<(WorkReport, Vec<OpaqueHash>)>,
    accd: &[OpaqueHash],
) -> Vec<(WorkReport, Vec<OpaqueHash>)> {
    accq.into_iter()
        .filter_map(|(report, deps)| {
            // Skip if the report's segment hash is already accumulated
            if accd.contains(&report.spec.hash) {
                return None;
            }

            // Remove accumulated dependencies
            let filtered_deps = deps
                .into_iter()
                .filter(|dep| !accd.contains(dep))
                .collect::<Vec<_>>();

            Some((report, filtered_deps))
        })
        .collect()
}

/// (Q) provides the sequence of work reports which are accumulatable given a set of
/// not yet accumulated work reports and their dependencies
fn priority(accq: Vec<(WorkReport, Vec<OpaqueHash>)>) -> Vec<WorkReport> {
    if accq.is_empty() {
        return vec![];
    }

    // splitting ready and pending work reports
    let (mut ready, mut pending) = (vec![], vec![]);
    for (report, deps) in accq {
        if deps.is_empty() {
            ready.push(report);
        } else {
            pending.push((report, deps));
        }
    }

    // recursively calling priority on the pending work reports
    //
    // TODO: use for-loop instead of recursion
    ready.extend(self::priority(self::edit(pending, &self::mapping(&ready))));
    ready
}

/// (P) extracts the corresponding work package hashes from a set of work reports
fn mapping(reports: &[WorkReport]) -> Vec<OpaqueHash> {
    reports.iter().map(|report| report.spec.hash).collect()
}
