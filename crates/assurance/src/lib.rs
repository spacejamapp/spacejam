//! Assurance is the process of ensuring that the results of a work-package are available to a super-majority of validators.

use {
    error::{Error, Result},
    score::{CORES_COUNT, VALIDATORS_SUPER_MAJORITY},
    state::{Input, Output, State},
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
        for window in input.assurances.windows(2) {
            if window[0].validator_index >= window[1].validator_index {
                return Err(Error::NotSortedOrUniqueAssurers);
            }
        }

        // Track assurance count per core
        let mut core_assurance_counts = vec![0u32; CORES_COUNT as usize];
        for assurance in input.assurances {
            // Count assurances per core
            for core_idx in 0..CORES_COUNT as usize {
                // TODO: workaround for the bitfield
                let assured = assurance.bitfield[0] >> (core_idx % 8) & 1;
                if assured == 0 {
                    continue;
                }

                // Validate the core has a pending report that hasn't timed out
                if let Some(assignment) = self.post_state.avail_assignments[core_idx].take() {
                    if input.slot <= assignment.timeout {
                        core_assurance_counts[core_idx] += 1;
                    }
                } else {
                    return Err(error::Error::CoreNotEngaged);
                }
            }
        }

        // Check which cores reached 2/3 majority
        let mut available_reports = Vec::new();
        for (core_idx, &assurance_count) in core_assurance_counts.iter().enumerate() {
            if assurance_count > VALIDATORS_SUPER_MAJORITY as u32 {
                if let Some(assignment) = &self.prev_state.avail_assignments[core_idx] {
                    available_reports.push(assignment.report.clone());
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
