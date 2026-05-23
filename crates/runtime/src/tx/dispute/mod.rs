//! Disputes extrinsic handler
//!
//! 1. update judgements on work-reports and validators (ψ)
//! 2. update pending reports (ρ)
use super::dispute;
use crypto::ed25519::SigItem;
pub use error::{Error, Result};
use score::{
    EPOCH_LENGTH, Ed25519Public, OpaqueHash, TimeSlot, VALIDATORS_COUNT, VALIDATORS_SUPER_MAJORITY,
    extrinsic::dispute::{Culprit, DisputesExtrinsic, DisputesRecords, Fault, Verdict},
    safrole::{ValidatorIter, ValidatorsData},
    service::AvailabilityAssignments,
};
use std::collections::{BTreeMap, HashSet};

pub mod error;

/// (ψ) Update disputes verdicts and offenders.
pub fn disputes(
    timeslot: TimeSlot,
    kappa: &ValidatorsData,
    lambda: &ValidatorsData,
    psi: DisputesRecords,
    extrinsic: &DisputesExtrinsic,
) -> Result<(DisputesRecords, DisputesRecords, Vec<SigItem>)> {
    let (mut records, mut triples) =
        dispute::verdicts(timeslot, kappa, lambda, &extrinsic.verdicts)?;

    // get validators for the current slot
    let validators: HashSet<Ed25519Public> = [kappa.ed25519(), lambda.ed25519()]
        .concat()
        .into_iter()
        .collect();

    // handle culprits
    let (culprit_offenders, culprit_triples) =
        dispute::culprits(&validators, &psi, &records.bad, &extrinsic.culprits)?;
    records.offenders.extend(&culprit_offenders);
    triples.extend(culprit_triples);

    // handle faults
    let (fault_offenders, fault_triples) =
        dispute::faults(&validators, &psi, &records.good, &extrinsic.faults)?;
    records.offenders.extend(&fault_offenders);
    triples.extend(fault_triples);

    let mut next_psi = psi;
    next_psi.good.extend(&records.good);
    next_psi.wonky.extend(&records.wonky);
    next_psi.bad.extend(&records.bad);
    // TODO: make offenders unique
    next_psi.offenders.extend(&records.offenders);
    next_psi.offenders.sort();

    Ok((next_psi, records, triples))
}

/// (ρ†) Update availability assignments based on verdicts (ψ')
pub fn reports(
    records: &DisputesRecords,
    assignments: &AvailabilityAssignments,
) -> AvailabilityAssignments {
    let mut next_assignments = assignments.clone();

    // Clean work-reports from rho if they were judged as uncertain or invalid
    for maybe_assignment in next_assignments.iter_mut() {
        if let Some(assignment) = maybe_assignment {
            let hashed = crypto::blake2b(&codec::encode(&assignment.report));

            // Clear if the report is in bad or wonky sets (i.e., t < ⌊2/3V⌋)
            if records.bad.contains(&hashed) || records.wonky.contains(&hashed) {
                *maybe_assignment = None;
            }
        }
    }
    next_assignments
}

// Update goodset, badset, wonkyset based on verdicts; collect sig triples.
fn verdicts(
    timeslot: TimeSlot,
    kappa: &ValidatorsData,
    lambda: &ValidatorsData,
    verdicts: &[Verdict],
) -> Result<(DisputesRecords, Vec<SigItem>)> {
    let mut records = DisputesRecords::default();
    let mut triples: Vec<SigItem> = Vec::new();
    let mut last_target: Option<OpaqueHash> = None;
    for verdict in verdicts {
        if verdict.votes.len() != VALIDATORS_SUPER_MAJORITY as usize {
            return Err(Error::NotEnoughValidators);
        }

        if let Some(last) = last_target
            && verdict.target <= last
        {
            return Err(Error::VerdictsNotSortedUnique);
        }
        last_target = Some(verdict.target);

        let mut aye = 0;
        let aye_message = verdict.signature_message(true);
        let nay_message = verdict.signature_message(false);
        let current_epoch = timeslot / EPOCH_LENGTH;
        let validators = if verdict.age >= current_epoch {
            kappa
        } else if verdict.age == current_epoch.saturating_sub(1) {
            lambda
        } else {
            return Err(Error::BadJudgementAge);
        };

        for (index, judgement) in verdict.votes.iter().enumerate() {
            if index != judgement.index as usize {
                return Err(Error::JudgementsNotSortedUnique);
            }

            let message = if judgement.vote {
                aye_message.clone()
            } else {
                nay_message.clone()
            };

            triples.push(SigItem {
                message,
                signature: judgement.signature,
                key: validators[judgement.index as usize].ed25519,
            });

            if judgement.vote {
                aye += 1;
            }
        }

        match aye {
            aye if aye == VALIDATORS_SUPER_MAJORITY => records.good.push(verdict.target),
            aye if aye == VALIDATORS_COUNT / 3 => records.wonky.push(verdict.target),
            0 => records.bad.push(verdict.target),
            _ => {
                tracing::error!("Bad vote split in verdict: {aye}/{VALIDATORS_SUPER_MAJORITY}");
                return Err(Error::BadVoteSplit);
            }
        }
    }

    Ok((records, triples))
}

/// (ψ) Update offenders based on culprits; collect sig triples.
fn culprits(
    validators: &HashSet<Ed25519Public>,
    records: &DisputesRecords,
    bad: &[OpaqueHash],
    culprits: &[Culprit],
) -> Result<(Vec<Ed25519Public>, Vec<SigItem>)> {
    let mut last_culprit = None;
    let mut bad_verdicts = bad.iter().map(|v| (v, 0)).collect::<BTreeMap<_, _>>();
    let mut offenders = vec![];
    let mut triples: Vec<SigItem> = Vec::new();

    for culprit in culprits {
        if !validators.contains(&culprit.key) {
            return Err(Error::BadGuarantorKey);
        }

        triples.push(SigItem {
            message: culprit.signature_message().to_vec(),
            signature: culprit.signature,
            key: culprit.key,
        });

        if records.good.contains(&culprit.target)
            || records.bad.contains(&culprit.target)
            || records.wonky.contains(&culprit.target)
        {
            return Err(Error::AlreadyJudged);
        }

        if records.offenders.contains(&culprit.key) {
            return Err(Error::OffenderAlreadyReported);
        }

        if let Some(last_culprit) = last_culprit
            && culprit < last_culprit
        {
            return Err(Error::CulpritsNotSortedUnique);
        }

        last_culprit = Some(culprit);
        if bad.contains(&culprit.target) {
            if let Some(count) = bad_verdicts.get_mut(&culprit.target) {
                *count += 1;
            }

            offenders.push(culprit.key);
        } else {
            return Err(Error::CulpritsVerdictNotBad);
        }
    }

    if bad_verdicts.iter().any(|(_, count)| *count < 2) {
        return Err(Error::NotEnoughCulprits);
    }

    Ok((offenders, triples))
}

/// (ψ) Update offenders based on faults; collect sig triples.
fn faults(
    validators: &HashSet<Ed25519Public>,
    records: &DisputesRecords,
    good: &[OpaqueHash],
    faults: &[Fault],
) -> Result<(Vec<Ed25519Public>, Vec<SigItem>)> {
    let mut last_fault = None;
    let mut verdicts = good.iter().map(|v| (v, 0)).collect::<BTreeMap<_, _>>();
    let mut new_offenders = vec![];
    let mut triples: Vec<SigItem> = Vec::new();

    for fault in faults {
        if !validators.contains(&fault.key) {
            return Err(Error::BadAuditorKey);
        }

        if records.good.contains(&fault.target)
            || records.bad.contains(&fault.target)
            || records.wonky.contains(&fault.target)
        {
            return Err(Error::AlreadyJudged);
        }

        if records.offenders.contains(&fault.key) {
            return Err(Error::OffenderAlreadyReported);
        }

        triples.push(SigItem {
            message: fault.singing_message(),
            signature: fault.signature,
            key: fault.key,
        });

        if let Some(last_fault) = last_fault
            && fault < last_fault
        {
            return Err(Error::FaultsNotSortedUnique);
        }

        last_fault = Some(fault);

        if (records.wonky.contains(&fault.target) && !records.good.contains(&fault.target))
            == fault.vote
        {
            if let Some(count) = verdicts.get_mut(&fault.target) {
                *count += 1;
            }

            new_offenders.push(fault.key);
        } else {
            return Err(Error::FaultVerdictWrong);
        }
    }

    if verdicts.iter().any(|(_, count)| *count < 1) {
        return Err(Error::NotEnoughFaults);
    }

    Ok((new_offenders, triples))
}
