//! Assurance is the process of ensuring that the results of a work-package are available to a super-majority of validators.

pub use error::{Error, Result};
use score::{
    extrinsic::AvailAssurance,
    safrole::ValidatorData,
    service::{AvailabilityAssignments, WorkReport},
    OpaqueHash, TimeSlot, CORES_COUNT, VALIDATORS_COUNT, VALIDATORS_SUPER_MAJORITY,
    WORK_REPORT_TIMEOUT_PERIOD,
};
use std::collections::HashSet;

mod error;

/// (ρ‡) Handle assurances input and return newly available reports (11.17)
pub fn reports(slot: TimeSlot, mut reports: AvailabilityAssignments) -> AvailabilityAssignments {
    for mb_report in reports.iter_mut() {
        if let Some(report) = mb_report {
            if slot >= report.timeout + WORK_REPORT_TIMEOUT_PERIOD {
                *mb_report = None;
            }
        }
    }

    reports
}

/// (W*) Handle assurances input and return newly available reports
pub fn available(
    reports: &AvailabilityAssignments,
    validators: &[ValidatorData],
    slot: TimeSlot,
    parent: OpaqueHash,
    assurances: &[AvailAssurance],
) -> Result<Vec<WorkReport>> {
    // Track assurance count per core
    let mut core_assurance_counts = [0u32; CORES_COUNT];
    let mut stale_reports = HashSet::new();

    // Check for stale reports
    for (core_idx, assignment) in reports.iter().enumerate() {
        if let Some(assignment) = assignment {
            if slot >= assignment.timeout + WORK_REPORT_TIMEOUT_PERIOD {
                stale_reports.insert(core_idx);
                continue;
            }
        }
    }

    // Check for engaged reports
    for (index, assurance) in assurances.iter().enumerate() {
        verify_assurance(validators, index as u16, assurance, parent)?;

        // Count assurances per core
        let bitsmap = assurance.bitsmap();
        for core_idx in 0..CORES_COUNT {
            if bitsmap[core_idx] == 0 {
                continue;
            }

            // Validate the core has a pending report that hasn't timed out
            //
            // TODO: check the maximum number of assurances per core
            if reports[core_idx].is_some() {
                core_assurance_counts[core_idx] += 1;
            } else if !stale_reports.contains(&core_idx) {
                return Err(error::Error::CoreNotEngaged);
            }
        }
    }

    // Check which cores reached 2/3 majority
    let mut available = Vec::new();
    for (core_idx, &assurance_count) in core_assurance_counts.iter().enumerate() {
        if assurance_count >= VALIDATORS_SUPER_MAJORITY as u32 {
            if let Some(assignment) = &reports[core_idx] {
                available.push(assignment.report.clone());
            }
        }
    }

    Ok(available)
}

/// Verifies the assurance
fn verify_assurance(
    validators: &[ValidatorData],
    index: u16,
    assurance: &AvailAssurance,
    parent: OpaqueHash,
) -> Result<()> {
    if assurance.validator_index >= VALIDATORS_COUNT {
        return Err(Error::BadValidatorIndex);
    }

    if assurance.validator_index != index {
        return Err(Error::NotSortedOrUniqueAssurers);
    }

    if assurance.anchor != parent {
        return Err(Error::BadAttestationParent);
    }

    if let Err(e) = validators[index as usize].verify_assurance(assurance) {
        tracing::warn!("assurance verification failed: {e}");
        return Err(Error::BadSignature);
    }

    Ok(())
}
