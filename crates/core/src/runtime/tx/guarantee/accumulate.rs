//! Accumulation of guarantees

use crate::{
    runtime::Storage,
    service::{AccumulatedQueue, ReadyQueue, WorkReport},
    OpaqueHash, TimeSlot,
};

/// (b) Accumulate the available work reports
pub fn accumulate(
    // The next timeslot
    _slot: TimeSlot,
    // The prior timeslot
    _tau: TimeSlot,
    // available work reports
    _reports: Vec<WorkReport>,
    // The ready queue (θ)
    _ready_queue: &ReadyQueue,
    // The accumulated queue (ξ)
    _accumulated_queue: &AccumulatedQueue,
    // The account storage
    _accounts: &impl Storage,
) -> anyhow::Result<OpaqueHash> {
    // (W_!) work reports to be accumulated immediately
    let _acci = ();

    // (W_Q) work reports to be queued for accumulation
    let _accq = ();

    edit();
    priority();
    mapping();

    Ok(Default::default())
}

/// (E) queue-editing function
fn edit() {}

/// (Q) Accumulation priority queue function
fn priority() {}

/// (P) Extracts the corresponding work-package hashes from a set of work reports
fn mapping() {}
