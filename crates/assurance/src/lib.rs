//! Assurance is the process of ensuring that the results of a work-package are available to a super-majority of validators.

use {
    error::{Error, Result},
    score::{CORES_COUNT, VALIDATORS_COUNT, VALIDATORS_SUPER_MAJORITY},
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
        // Validate assurances are ordered by validator index
        for (index, assurance) in input.assurances.iter().enumerate() {
            if assurance.validator_index >= VALIDATORS_COUNT {
                return Err(Error::BadValidatorIndex);
            }

            if assurance.validator_index != index as u16 {
                return Err(Error::NotSortedOrUniqueAssurers);
            }

            if assurance.anchor != input.parent {
                return Err(Error::BadAttestationParent);
            }
        }

        // Track assurance count per core
        let mut core_assurance_counts = vec![0u32; CORES_COUNT as usize];
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
        for assurance in input.assurances {
            // Count assurances per core
            for core_idx in 0..CORES_COUNT as usize {
                let assured = assurance.bitfield[core_idx / 8] >> (core_idx % 8) & 1;
                if assured == 0 {
                    continue;
                }

                // Validate the core has a pending report that hasn't timed out
                if self.post_state.avail_assignments[core_idx].is_some() {
                    core_assurance_counts[core_idx] += 1;
                } else {
                    if !stale_reports.contains_key(&core_idx) {
                        return Err(error::Error::CoreNotEngaged);
                    }
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
}

impl Handler {
    pub fn from(state: State) -> Self {
        Self {
            prev_state: state.clone(),
            post_state: state,
        }
    }
}
