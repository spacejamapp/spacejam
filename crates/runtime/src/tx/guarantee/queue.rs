//! Queue of work reports

use score::{
    service::{AccumulatedQueue, ReadyQueue, ReadyReport, WorkReport},
    OpaqueHash, TimeSlot,
};

/// Extracts the accumulatable work reports
///
/// ref GP: 12.1 History and Queuing
pub fn accumulatable(
    slot: TimeSlot,
    reports: Vec<WorkReport>,
    ready_queue: &ReadyQueue,
    accumulated_queue: &AccumulatedQueue,
) -> (Vec<WorkReport>, Vec<ReadyReport>) {
    let accumulated = accumulated_queue
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();

    // (W_!) work reports to be accumulated immediately
    let (ready, pending): (Vec<_>, Vec<_>) = reports
        .into_iter()
        .partition(|report| report.is_immediate());

    // (W_Q) work reports to be queued for accumulation
    let pending: Vec<ReadyReport> = self::pairing(pending, &accumulated);
    let idx = (slot % score::EPOCH_LENGTH) as usize;

    // extract the work package hashes
    let queue = self::priority(self::edit(
        [
            ready_queue[idx..].iter().flatten().cloned().collect(),
            ready_queue[..idx].iter().flatten().cloned().collect(),
            pending.clone(),
        ]
        .concat(),
        &self::mapping(&ready),
    ));

    ([ready, queue].concat(), pending)
}

/// (D) pairing work reports with their dependencies
fn pairing(pending: Vec<WorkReport>, accumulated: &[OpaqueHash]) -> Vec<ReadyReport> {
    self::edit(
        pending
            .into_iter()
            .map(|report| {
                let deps = report
                    .context
                    .prerequisites
                    .iter()
                    .cloned()
                    .chain(report.lookup.keys().copied())
                    .collect::<Vec<_>>();

                ReadyReport {
                    report,
                    dependencies: deps,
                }
            })
            .collect(),
        accumulated,
    )
}

/// (E) queue-editing function
///
/// Removes the accumulated dependencies from the accumulated queue
pub fn edit(pending: Vec<ReadyReport>, accumulated: &[OpaqueHash]) -> Vec<ReadyReport> {
    pending
        .into_iter()
        .filter_map(|report| {
            // Skip if the report's segment hash is already accumulated
            if accumulated.contains(&report.report.spec.hash) {
                return None;
            }

            // Remove accumulated dependencies
            let filtered_deps = report
                .dependencies
                .into_iter()
                .filter(|dep| !accumulated.contains(dep))
                .collect::<Vec<_>>();

            Some(ReadyReport {
                report: report.report,
                dependencies: filtered_deps,
            })
        })
        .collect()
}

/// (Q) provides the sequence of work reports which are accumulatable given a set of
/// not yet accumulated work reports and their dependencies
fn priority(rpending: Vec<ReadyReport>) -> Vec<WorkReport> {
    if rpending.is_empty() {
        return vec![];
    }

    // splitting ready and pending work reports
    let (mut ready, mut pending) = (vec![], vec![]);
    for report in rpending {
        if report.dependencies.is_empty() {
            ready.push(report.report);
        } else {
            pending.push(report);
        }
    }

    // If we have nothing ready initially, we can't make progress
    if ready.is_empty() {
        return vec![];
    }

    // Iteratively process pending reports until no more progress can be made
    while !pending.is_empty() {
        // Remove accumulated dependencies
        pending = self::edit(pending, &self::mapping(&ready));

        // Move ready items from pending to ready
        let (nready, npending): (Vec<_>, Vec<_>) = pending
            .into_iter()
            .partition(|report| report.dependencies.is_empty());

        // If we have nothing ready, we can't make progress
        if nready.is_empty() {
            break;
        }

        ready.extend(nready.into_iter().map(|r| r.report));
        pending = npending;
    }

    ready
}

/// (P) extracts the corresponding work package hashes from a set of work reports
fn mapping(reports: &[WorkReport]) -> Vec<OpaqueHash> {
    reports.iter().map(|report| report.spec.hash).collect()
}
