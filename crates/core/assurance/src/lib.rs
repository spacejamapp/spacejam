//! Assurance is the process of ensuring that the results of a work-package are available to a super-majority of validators.

use {
    error::{Error, Result},
    score::{
        extrinsic::AvailAssurance, OpaqueHash, CORES_COUNT, VALIDATORS_COUNT,
        VALIDATORS_SUPER_MAJORITY,
    },
    state::{Input, Output, State},
    std::collections::BTreeMap,
};

pub mod error;
pub mod state;

/// Handler processes assurances for work reports
pub struct Handler {
    pub prev_state: State,
    pub post_state: State,
}

impl Handler {
    /// Handle assurances input and return newly available reports
    pub fn handle(&mut self, input: Input) -> Result<Output> {
        // Track assurance count per core
        let mut core_assurance_counts = [0u32; CORES_COUNT];
        let mut available_reports = Vec::new();
        let mut stale_reports = BTreeMap::new();

        // Check for stale reports
        for (core_idx, assignment) in self.prev_state.avail_assignments.iter().enumerate() {
            if let Some(assignment) = assignment {
                if input.slot <= assignment.timeout + 1 {
                    continue;
                }

                self.post_state.avail_assignments[core_idx] = None;
                stale_reports.insert(core_idx, ());

                if !input.assurances.is_empty() {
                    available_reports.push(assignment.report.clone());
                }
            }
        }

        // Check for engaged reports
        for (index, assurance) in input.assurances.iter().enumerate() {
            self.verify_assurance(index as u16, assurance, input.parent)?;

            // Count assurances per core
            let bitsmap = assurance.bitsmap();
            for core_idx in 0..CORES_COUNT {
                if bitsmap[core_idx] == 0 {
                    continue;
                }

                // Validate the core has a pending report that hasn't timed out
                if self.post_state.avail_assignments[core_idx].is_some() {
                    core_assurance_counts[core_idx] += 1;
                } else if !stale_reports.contains_key(&core_idx) {
                    return Err(error::Error::CoreNotEngaged);
                }
            }
        }

        // Check which cores reached 2/3 majority
        for (core_idx, &assurance_count) in core_assurance_counts.iter().enumerate() {
            if assurance_count >= VALIDATORS_SUPER_MAJORITY as u32 {
                if let Some(assignment) = self.post_state.avail_assignments[core_idx].take() {
                    available_reports.push(assignment.report);
                }
            }
        }

        Ok(Output {
            reported: available_reports,
        })
    }

    /// Verifies the assurance
    fn verify_assurance(
        &self,
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

        if let Err(e) = self.prev_state.curr_validators[index as usize].verify_assurance(assurance)
        {
            tracing::warn!("assurance verification failed: {e}");
            return Err(Error::BadSignature);
        }

        Ok(())
    }
}

impl Handler {
    pub fn from(state: State) -> Self {
        Self {
            prev_state: state.clone(),
            post_state: state,
        }
    }
}
