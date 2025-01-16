//! Assurance is the process of ensuring that the results of a work-package are available to a super-majority of validators.

pub use {
    error::{Error, Result},
    state::{State, StateJson},
};
use {
    score::{
        extrinsic::AvailAssurance, work::WorkReport, Block, OpaqueHash, CORES_COUNT,
        VALIDATORS_COUNT, VALIDATORS_SUPER_MAJORITY,
    },
    std::collections::BTreeMap,
};

mod error;
mod state;

/// Validate assurance extrinsics
pub fn validate(state: &mut score::State, block: &Block) -> Result<Vec<WorkReport>> {
    let mut pstate = State::from(&*state);
    let reports = pstate.validate(block)?;
    pstate.apply(state);
    Ok(reports)
}

impl State {
    /// Handle assurances input and return newly available reports
    fn validate(&mut self, block: &Block) -> Result<Vec<WorkReport>> {
        let slot = block.header.slot;
        let parent = block.header.parent;
        let assurances = block.extrinsic.assurances.clone();

        // Track assurance count per core
        let mut core_assurance_counts = [0u32; CORES_COUNT];
        let mut available_reports = Vec::new();
        let mut stale_reports = BTreeMap::new();

        // Check for stale reports
        for (core_idx, assignment) in self.avail_assignments.clone().iter().enumerate() {
            if let Some(assignment) = assignment {
                if slot <= assignment.timeout + 1 {
                    continue;
                }

                self.avail_assignments[core_idx] = None;
                stale_reports.insert(core_idx, ());

                if !assurances.is_empty() {
                    available_reports.push(assignment.report.clone());
                }
            }
        }

        // Check for engaged reports
        for (index, assurance) in assurances.iter().enumerate() {
            self.verify_assurance(index as u16, assurance, parent)?;

            // Count assurances per core
            let bitsmap = assurance.bitsmap();
            for core_idx in 0..CORES_COUNT {
                if bitsmap[core_idx] == 0 {
                    continue;
                }

                // Validate the core has a pending report that hasn't timed out
                if self.avail_assignments[core_idx].is_some() {
                    core_assurance_counts[core_idx] += 1;
                } else if !stale_reports.contains_key(&core_idx) {
                    return Err(error::Error::CoreNotEngaged);
                }
            }
        }

        // Check which cores reached 2/3 majority
        for (core_idx, &assurance_count) in core_assurance_counts.iter().enumerate() {
            if assurance_count >= VALIDATORS_SUPER_MAJORITY as u32 {
                if let Some(assignment) = self.avail_assignments[core_idx].take() {
                    available_reports.push(assignment.report);
                }
            }
        }

        Ok(available_reports)
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

        if let Err(e) = self.curr_validators[index as usize].verify_assurance(assurance) {
            tracing::warn!("assurance verification failed: {e}");
            return Err(Error::BadSignature);
        }

        Ok(())
    }
}
