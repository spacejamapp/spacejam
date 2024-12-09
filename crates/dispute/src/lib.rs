use std::collections::BTreeMap;

use codec::Json;
use error::{Error, Result};
use score::{
    dispute::{DisputesExtrinsic, DisputesRecords, DisputesRecordsJson},
    misc::{Ed25519Public, TimeSlot, ValidatorDataJson, ValidatorsData},
    work::{AvailabilityAssignment, AvailabilityAssignmentJson},
    VALIDATORS_COUNT, VALIDATORS_SUPER_MAJORITY,
};
use serde::{Deserialize, Serialize};

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
}

#[derive(Json, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct OffendersMark {
    /// [H_o] Offenders marker
    #[json(Vec<String>)]
    offenders_mark: Vec<Ed25519Public>,
}

impl DisputesHandler {
    /// Handle an extrinsic
    pub fn handle(&mut self, extrinsic: DisputesExtrinsic) -> Result<OffendersMark> {
        // let prev_epoch = self.state.tau / EPOCH_LENGTH;

        // Update goodset, badset, wonkyset based on verdicts
        for verdict in extrinsic.verdicts {
            if verdict.votes.len() != VALIDATORS_SUPER_MAJORITY as usize {
                return Err(Error::NotEnoughValidators);
            }

            if self.next_state.psi.good.contains(&verdict.target)
                || self.next_state.psi.bad.contains(&verdict.target)
                || self.next_state.psi.wonky.contains(&verdict.target)
            {
                return Err(Error::AlreadyJudged);
            }

            let mut aye = 0;
            for judgement in verdict.votes {
                let aye_message = verdict.signature_message(true);
                let nay_message = verdict.signature_message(false);
                if let Err(e) = crypto::ed25519::verify(
                    if judgement.vote {
                        &aye_message
                    } else {
                        &nay_message
                    },
                    judgement.signature,
                    self.state.kappa[judgement.index as usize].ed25519,
                ) {
                    tracing::error!("Invalid signature in verdict: {e}");
                    self.next_state = self.state.clone();
                    return Err(Error::BadSignature);
                }

                if judgement.vote {
                    aye += 1;
                }
            }

            match aye {
                aye if aye == (2 * VALIDATORS_COUNT as usize / 3) + 1 => {
                    self.next_state.psi.good.push(verdict.target);
                }
                aye if aye == 0 => {
                    self.next_state.psi.bad.push(verdict.target);
                }
                aye if aye == VALIDATORS_COUNT as usize / 3 => {
                    self.next_state.psi.wonky.push(verdict.target);
                }
                _ => {}
            }
        }

        let mut offenders_mark = vec![];
        let mut last_culprit = None;
        let mut bad_verdicts = self
            .next_state
            .psi
            .bad
            .clone()
            .into_iter()
            .map(|v| (v, 0))
            .collect::<BTreeMap<_, _>>();

        for culprit in extrinsic.culprits {
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
            if let Err(e) = culprit.verify() {
                tracing::error!("Invalid signature in culprit: {e}");
                self.next_state = self.state.clone();
                return Err(Error::BadSignature);
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

        for fault in extrinsic.faults {
            if self.next_state.psi.offenders.contains(&fault.key) {
                self.next_state = self.state.clone();
                return Err(Error::OffenderAlreadyReported);
            }

            if let Err(e) = fault.verify() {
                tracing::error!("Invalid signature in fault: {e}");
                self.next_state = self.state.clone();
                return Err(Error::BadSignature);
            }

            if (self.next_state.psi.bad.contains(&fault.target)
                && !self.state.psi.good.contains(&fault.target))
                == fault.vote
            {
                self.next_state.psi.offenders.push(fault.key);
                offenders_mark.push(fault.key);
            }
        }

        // Clear work-reports from rho if they were judged as uncertain or invalid
        // This implements equation (eq:removenonpositive) from the graypaper
        for (_, maybe_assignment) in self.next_state.rho.iter_mut().enumerate() {
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

        Ok(OffendersMark { offenders_mark })
    }
}

impl From<State> for DisputesHandler {
    fn from(state: State) -> Self {
        Self {
            next_state: state.clone(),
            state,
        }
    }
}
