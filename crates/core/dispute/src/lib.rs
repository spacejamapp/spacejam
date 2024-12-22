use error::{Error, Result};
use score::{
    extrinsic::dispute::{
        Culprit, DisputesExtrinsic, DisputesRecords, DisputesRecordsJson, Fault, Verdict,
    },
    validator::{ValidatorDataJson, ValidatorsData},
    work::{AvailabilityAssignment, AvailabilityAssignmentJson},
    Ed25519Public, TimeSlot, EPOCH_LENGTH, VALIDATORS_COUNT, VALIDATORS_SUPER_MAJORITY,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::collections::BTreeMap;

pub mod error;

#[derive(Debug, PartialEq, Eq, Json, Serialize, Deserialize, Clone)]
pub struct State {
    /// [ψ] Disputes verdicts and offenders
    #[json(nested)]
    pub psi: DisputesRecords,
    /// [ρ] Availability cores assignments
    #[json(Vec<Option<AvailabilityAssignmentJson>>)]
    pub rho: Vec<Option<AvailabilityAssignment>>,
    /// [τ] Timeslot
    pub tau: TimeSlot,
    /// [κ] Validators active in the current epoch
    #[json(Vec<ValidatorDataJson>)]
    pub kappa: ValidatorsData,
    /// [λ] Validators active in the previous epoch
    #[json(Vec<ValidatorDataJson>)]
    pub lambda: ValidatorsData,
}

/// Disputes handler
pub struct DisputesHandler {
    pub state: State,
    pub next_state: State,
    pub records: DisputesRecords,
}

#[derive(Json, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct OffendersMark {
    /// [H_o] Offenders marker
    #[json(Vec<String>)]
    pub offenders_mark: Vec<Ed25519Public>,
}

impl DisputesHandler {
    /// Handle an extrinsic
    pub fn handle(&mut self, extrinsic: DisputesExtrinsic) -> Result<OffendersMark> {
        self.handle_verdicts(extrinsic.verdicts)?;

        let mut offenders_mark = vec![];
        self.handle_culprits(&mut offenders_mark, extrinsic.culprits)?;
        self.handle_faults(&mut offenders_mark, extrinsic.faults)?;

        // Clear work-reports from rho if they were judged as uncertain or invalid
        // This implements equation (eq:removenonpositive) from the graypaper
        for maybe_assignment in self.next_state.rho.iter_mut() {
            if let Some(assignment) = maybe_assignment {
                let hashed =
                    crypto::blake2b(&codec::encode(&assignment.report).expect("failed to encode "));
                // Clear if the report is in bad or wonky sets (i.e., t < ⌊2/3V⌋)
                if self.next_state.psi.bad.contains(&hashed)
                    || self.next_state.psi.wonky.contains(&hashed)
                {
                    *maybe_assignment = None;
                }
            }
        }

        self.next_state.psi.offenders.sort();
        Ok(OffendersMark { offenders_mark })
    }

    // Update goodset, badset, wonkyset based on verdicts
    fn handle_verdicts(&mut self, verdicts: Vec<Verdict>) -> Result<()> {
        let mut last_verdict = None;
        for verdict in verdicts {
            if verdict.votes.len() != VALIDATORS_SUPER_MAJORITY as usize {
                return Err(Error::NotEnoughValidators);
            }

            /*  if verdict.age < self.state.tau {
                return Err(Error::BadJudgementAge);
            } */

            if let Some(last_verdict) = last_verdict.take() {
                if verdict < last_verdict {
                    self.next_state = self.state.clone();
                    return Err(Error::VerdictsNotSortedUnique);
                }
            } else {
                last_verdict = Some(verdict.clone());
            }

            let mut aye = 0;
            let aye_message = verdict.signature_message(true);
            let nay_message = verdict.signature_message(false);
            for (index, judgement) in verdict.votes.iter().enumerate() {
                if index != judgement.index as usize {
                    self.next_state = self.state.clone();
                    return Err(Error::JudgementsNotSortedUnique);
                }

                let message = if judgement.vote {
                    &aye_message
                } else {
                    &nay_message
                };

                let current_epoch = self.state.tau / EPOCH_LENGTH;
                if verdict.age >= current_epoch {
                    if let Err(e) = crypto::ed25519::verify(
                        message,
                        judgement.signature,
                        self.state.kappa[judgement.index as usize].ed25519,
                    ) {
                        tracing::warn!("Invalid verdict signature for judgement {index}: {e}");
                        self.next_state = self.state.clone();
                        return Err(Error::BadSignature);
                    }
                } else if verdict.age == current_epoch.saturating_sub(1) {
                    if let Err(e) = crypto::ed25519::verify(
                        message,
                        judgement.signature,
                        self.state.lambda[judgement.index as usize].ed25519,
                    ) {
                        tracing::warn!("Invalid verdict signature for judgement {index}: {e}");
                        self.next_state = self.state.clone();
                        return Err(Error::BadSignature);
                    }
                } else {
                    return Err(Error::BadJudgementAge);
                }

                if judgement.vote {
                    aye += 1;
                }
            }

            match aye {
                aye if aye == VALIDATORS_SUPER_MAJORITY => {
                    self.records.good.push(verdict.target);
                    self.next_state.psi.good.push(verdict.target);
                }
                aye if aye == VALIDATORS_COUNT / 3 => {
                    self.records.wonky.push(verdict.target);
                    self.next_state.psi.wonky.push(verdict.target);
                }
                0 => {
                    self.records.bad.push(verdict.target);
                    self.next_state.psi.bad.push(verdict.target);
                }
                _ => {
                    tracing::error!("Bad vote split in verdict: {aye}/{VALIDATORS_SUPER_MAJORITY}");
                    self.next_state = self.state.clone();
                    return Err(Error::BadVoteSplit);
                }
            }
        }

        Ok(())
    }

    fn handle_culprits(
        &mut self,
        offenders_mark: &mut Vec<Ed25519Public>,
        culprits: Vec<Culprit>,
    ) -> Result<()> {
        let mut last_culprit = None;
        let mut bad_verdicts = self
            .records
            .bad
            .clone()
            .into_iter()
            .map(|v| (v, 0))
            .collect::<BTreeMap<_, _>>();

        for culprit in culprits {
            if let Err(e) = culprit.verify() {
                tracing::error!("Invalid signature in culprit: {e}");
                self.next_state = self.state.clone();
                return Err(Error::BadSignature);
            }

            if self.state.psi.good.contains(&culprit.target)
                || self.state.psi.bad.contains(&culprit.target)
                || self.state.psi.wonky.contains(&culprit.target)
            {
                self.next_state = self.state.clone();
                return Err(Error::AlreadyJudged);
            }

            if self.next_state.psi.offenders.contains(&culprit.key) {
                self.next_state = self.state.clone();
                return Err(Error::OffenderAlreadyReported);
            }

            if !self.next_state.psi.bad.contains(&culprit.target) {
                self.next_state = self.state.clone();
                return Err(Error::CulpritsVerdictNotBad);
            }

            if let Some(last_culprit) = last_culprit {
                if culprit < last_culprit {
                    self.next_state = self.state.clone();
                    return Err(Error::CulpritsNotSortedUnique);
                }
            }

            last_culprit = Some(culprit.clone());
            if self.next_state.psi.bad.contains(&culprit.target) {
                if let Some(count) = bad_verdicts.get_mut(&culprit.target) {
                    *count += 1;
                }

                self.next_state.psi.offenders.push(culprit.key);
                offenders_mark.push(culprit.key);
            }
        }

        if bad_verdicts.iter().any(|(_, count)| *count < 2) {
            self.next_state = self.state.clone();
            return Err(Error::NotEnoughCulprits);
        }

        Ok(())
    }

    fn handle_faults(
        &mut self,
        offenders_mark: &mut Vec<Ed25519Public>,
        faults: Vec<Fault>,
    ) -> Result<()> {
        let mut last_fault = None;
        let mut verdicts = self
            .records
            .good
            .clone()
            .into_iter()
            .map(|v| (v, 0))
            .collect::<BTreeMap<_, _>>();

        for fault in faults {
            if self.state.psi.good.contains(&fault.target)
                || self.state.psi.bad.contains(&fault.target)
                || self.state.psi.wonky.contains(&fault.target)
            {
                self.next_state = self.state.clone();
                return Err(Error::AlreadyJudged);
            }

            if self.next_state.psi.offenders.contains(&fault.key) {
                self.next_state = self.state.clone();
                return Err(Error::OffenderAlreadyReported);
            }

            if let Err(e) = fault.verify() {
                tracing::error!("Invalid signature in fault: {e}");
                self.next_state = self.state.clone();
                return Err(Error::BadSignature);
            }

            if let Some(last_fault) = last_fault {
                if fault < last_fault {
                    self.next_state = self.state.clone();
                    return Err(Error::FaultsNotSortedUnique);
                }
            }

            last_fault = Some(fault.clone());

            if (self.next_state.psi.wonky.contains(&fault.target)
                && !self.state.psi.good.contains(&fault.target))
                == fault.vote
            {
                if let Some(count) = verdicts.get_mut(&fault.target) {
                    *count += 1;
                }

                self.next_state.psi.offenders.push(fault.key);
                offenders_mark.push(fault.key);
            } else {
                self.next_state = self.state.clone();
                return Err(Error::FaultVerdictWrong);
            }
        }

        if verdicts.iter().any(|(_, count)| *count < 1) {
            self.next_state = self.state.clone();
            return Err(Error::NotEnoughFaults);
        }

        Ok(())
    }
}

impl From<State> for DisputesHandler {
    fn from(state: State) -> Self {
        Self {
            next_state: state.clone(),
            state,
            records: Default::default(),
        }
    }
}
