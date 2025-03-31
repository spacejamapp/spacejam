//! Queue of work reports

use crate::{
    service::{AccumulatedQueue, ReadyQueue, ReadyReport, WorkReport},
    OpaqueHash, TimeSlot,
};

/// Extracts the accumulatable work reports
///
/// ref GP: 12.1 Hisotry and Queuing
pub fn accumulatable(
    slot: TimeSlot,
    reports: Vec<WorkReport>,
    ready_queue: &ReadyQueue,
    accumulated_queue: &AccumulatedQueue,
) -> (Vec<WorkReport>, Vec<ReadyReport>) {
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
    let accq: Vec<ReadyReport> = self::pairing(accq, &accd);
    let idx = (slot % crate::EPOCH_LENGTH) as usize;

    // extract the work package hashes
    (
        self::priority(self::edit(
            [
                ready_queue[idx..].iter().flatten().cloned().collect(),
                ready_queue[..idx].iter().flatten().cloned().collect(),
                accq.clone(),
            ]
            .concat(),
            &self::mapping(&acci),
        )),
        accq,
    )
}

/// (D) pairing work reports with their dependencies
fn pairing(accq: Vec<WorkReport>, accd: &[OpaqueHash]) -> Vec<ReadyReport> {
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

                ReadyReport {
                    report,
                    dependencies: deps,
                }
            })
            .collect(),
        accd,
    )
}

/// (E) queue-editing function
///
/// Removes the accumulated dependencies from the accumulated queue
pub fn edit(accq: Vec<ReadyReport>, accd: &[OpaqueHash]) -> Vec<ReadyReport> {
    accq.into_iter()
        .filter_map(|report| {
            // Skip if the report's segment hash is already accumulated
            if accd.contains(&report.report.spec.hash) {
                return None;
            }

            // Remove accumulated dependencies
            let filtered_deps = report
                .dependencies
                .into_iter()
                .filter(|dep| !accd.contains(dep))
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
fn priority(accq: Vec<ReadyReport>) -> Vec<WorkReport> {
    if accq.is_empty() {
        return vec![];
    }

    println!("accq: {:?}", accq.len());

    // splitting ready and pending work reports
    let (mut ready, mut pending) = (vec![], vec![]);
    for report in accq {
        if report.dependencies.is_empty() {
            ready.push(report.report);
        } else {
            pending.push(report);
        }
    }

    // recursively calling priority on the pending work reports
    //
    // TODO: use for-loop instead of recursion (dead loop ?)
    ready.extend(self::priority(self::edit(pending, &self::mapping(&ready))));
    ready
}

/// (P) extracts the corresponding work package hashes from a set of work reports
fn mapping(reports: &[WorkReport]) -> Vec<OpaqueHash> {
    reports.iter().map(|report| report.spec.hash).collect()
}
