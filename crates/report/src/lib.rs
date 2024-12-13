//! Reporting is the process of reporting the results of a work-package to the service state singleton.

use {
    error::{Error, Result},
    score::{
        extrinsic::ValidatorSignature, work::AvailabilityAssignment, CORES_COUNT,
        VALIDATORS_SUPER_MAJORITY,
    },
    state::{Input, Output, ReportedPackage, State},
};

pub mod error;
pub mod state;

/// Handler of the reporting module.
pub struct Handler {
    pub prev: State,
    pub next: State,
}

impl Handler {
    /// Handle work reports according to the guarantees extrinsic
    pub fn handle(&mut self, input: Input) -> Result<Output> {
        let mut reported = Vec::new();
        let mut reporters = Vec::new();

        // Validate guarantees are ordered by core index
        for window in input.guarantees.windows(2) {
            if window[0].report.core_index >= window[1].report.core_index {
                return Err(Error::NotSortedOrUniqueGuarantors);
            }
        }

        // Process each guarantee
        for guarantee in input.guarantees {
            let core_idx = guarantee.report.core_index as usize;

            // Validate core index
            if core_idx >= CORES_COUNT {
                return Err(Error::BadCoreIndex);
            }

            // Check if core already has a pending report that hasn't timed out
            if let Some(assignment) = &self.next.avail_assignments[core_idx] {
                if input.slot <= assignment.timeout {
                    return Err(Error::CoreEngaged);
                }
            }

            // Validate report's authorizer is in the authorization pool
            if !self.next.auth_pools[core_idx].contains(&guarantee.report.authorizer_hash) {
                return Err(Error::CoreUnauthorized);
            }

            // Require at least 2/3 guarantors
            if guarantee.signatures.len() < VALIDATORS_SUPER_MAJORITY as usize {
                return Err(Error::InsufficientGuarantees);
            }

            // Validate guarantors' signatures
            for ValidatorSignature {
                validator_index,
                signature: _,
            } in &guarantee.signatures
            {
                let validator_index = *validator_index as usize;
                if validator_index >= self.next.curr_validators.len() {
                    return Err(Error::BadValidatorIndex);
                }

                // TODO: implement signature verification
                /* if !self.next.curr_validators[validator_index]
                    .verify_guarantee(&guarantee.report, &signature)
                {
                    return Err(Error::BadSignature);
                } */
            }

            // Record reported package
            reported.push(ReportedPackage {
                work_package_hash: guarantee.report.package_spec.hash,
                segment_tree_root: guarantee
                    .report
                    .segment_root_lookup
                    .first()
                    .ok_or(Error::SegmentRootLookupInvalid)?
                    .segment_tree_root,
            });

            // Create availability assignment
            let assignment = AvailabilityAssignment {
                report: guarantee.report,
                timeout: input.slot + 5, // Reports timeout after 5 slots
            };

            // Update state
            self.next.avail_assignments[core_idx] = Some(assignment);

            // Record reporters (guarantors)
            reporters.extend(guarantee.signatures.iter().map(|sig| {
                self.next.curr_validators[sig.validator_index as usize]
                    .ed25519
                    .clone()
            }));
        }

        Ok(Output {
            reported,
            reporters,
        })
    }
}

impl From<State> for Handler {
    fn from(state: State) -> Self {
        Self {
            prev: state.clone(),
            next: state,
        }
    }
}
