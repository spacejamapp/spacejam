//! Assurance is the process of ensuring that the results of a work-package are available to a super-majority of validators.

pub use error::{Error, Result};
use score::{
    CORES_COUNT, OpaqueHash, TimeSlot, VALIDATORS_COUNT, VALIDATORS_SUPER_MAJORITY,
    WORK_REPORT_TIMEOUT_PERIOD,
    extrinsic::AvailAssurance,
    safrole::ValidatorData,
    service::{AvailabilityAssignments, WorkReport},
};

mod error;

/// (ρ‡) Handle assurances input and return newly available reports (11.17)
pub fn reports(
    slot: TimeSlot,
    available: &[WorkReport],
    mut reports: AvailabilityAssignments,
) -> AvailabilityAssignments {
    for mb_report in reports.iter_mut() {
        if let Some(report) = mb_report
            && (available.contains(&report.report)
                || slot >= report.timeout + WORK_REPORT_TIMEOUT_PERIOD)
        {
            *mb_report = None;
        }
    }

    reports
}

/// (W) Handle assurances input and return newly available reports
pub fn available(
    reports: &AvailabilityAssignments,
    validators: &[ValidatorData],
    parent: OpaqueHash,
    assurances: &[AvailAssurance],
) -> Result<(Vec<WorkReport>, [u32; CORES_COUNT])> {
    // Track assurance count per core
    let mut core_assurance_counts = [0u32; CORES_COUNT];

    // Check for engaged reports: cheap checks first, then batch verify sigs.
    let mut assuror = None;
    for assurance in assurances.iter() {
        if assurance.validator_index >= VALIDATORS_COUNT {
            return Err(Error::BadValidatorIndex);
        }
        if assurance.anchor != parent {
            return Err(Error::BadAttestationParent);
        }
        if let Some(last) = assuror
            && assurance.validator_index <= last
        {
            return Err(Error::NotSortedOrUniqueAssurers);
        }
        assuror = Some(assurance.validator_index);

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
            } else {
                return Err(error::Error::CoreNotEngaged);
            }
        }
    }

    let messages: Vec<Vec<u8>> = assurances.iter().map(|a| a.singing_message()).collect();
    let verify_items: Vec<_> = assurances
        .iter()
        .zip(messages.iter())
        .map(|(a, m)| {
            (
                m.as_slice(),
                a.signature,
                validators[a.validator_index as usize].ed25519,
            )
        })
        .collect();
    crypto::ed25519::batch_verify(&verify_items).map_err(|_| Error::BadSignature)?;

    // Check which cores reached 2/3 majority
    let mut available = Vec::new();
    for (core_idx, &assurance_count) in core_assurance_counts.iter().enumerate() {
        if assurance_count >= VALIDATORS_SUPER_MAJORITY as u32
            && let Some(assignment) = &reports[core_idx]
        {
            available.push(assignment.report.clone());
        }
    }

    Ok((available, core_assurance_counts))
}
