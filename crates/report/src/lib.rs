//! Reporting is the process of reporting the results of a work-package to the service state singleton.

use {
    error::{Error, Result},
    score::{
        extrinsic::ReportGuarantee, misc::TimeSlot, work::AvailabilityAssignment, CORES_COUNT,
        VALIDATORS_COUNT, VALIDATORS_SUPER_MAJORITY,
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
        let code_hashes = self
            .prev
            .services
            .iter()
            .map(|s| s.info.code_hash)
            .collect::<Vec<_>>();
        let service_ids = self.prev.services.iter().map(|s| s.id).collect::<Vec<_>>();

        // Process each guarantee
        for (core_index, guarantee) in input.guarantees.into_iter().enumerate() {
            self.validate_core(input.slot, core_index, &guarantee)?;

            if guarantee
                .report
                .results
                .iter()
                .any(|r| !code_hashes.contains(&r.code_hash))
            {
                return Err(Error::BadCodeHash);
            }

            if guarantee
                .report
                .results
                .iter()
                .any(|r| !service_ids.contains(&r.service_id))
            {
                return Err(Error::BadServiceId);
            }

            self.validate_block(&guarantee)?;
            self.validate_signatures(&guarantee)?;

            // Validate report's authorizer is in the authorization pool
            if !self.next.auth_pools[core_index].contains(&guarantee.report.authorizer_hash) {
                return Err(Error::CoreUnauthorized);
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
            self.next.avail_assignments[core_index] = Some(assignment);

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

    fn validate_block(&self, guarantee: &ReportGuarantee) -> Result<()> {
        let Some(block) = self
            .prev
            .recent_blocks
            .iter()
            .find(|b| b.header_hash == guarantee.report.context.anchor)
        else {
            return Err(Error::AnchorNotRecent);
        };

        // Validate state root
        if block.state_root != guarantee.report.context.state_root {
            return Err(Error::BadStateRoot);
        }

        // Validate beefy mmr root
        //
        // FIXME: This verification could be wrong.
        /* if block
            .mmr
            .peaks
            .get(guarantee.report.context.lookup_anchor_slot as usize)
            .cloned()
            .flatten()
            .ok_or(Error::BadBeefyMmrRoot)?
            != guarantee.report.context.beefy_root
        {
            return Err(Error::BadBeefyMmrRoot);
        } */
        Ok(())
    }

    /// Validate core assignments
    fn validate_core(
        &self,
        slot: TimeSlot,
        core_index: usize,
        guarantee: &ReportGuarantee,
    ) -> Result<()> {
        if guarantee.report.core_index >= CORES_COUNT as u16 {
            return Err(Error::BadCoreIndex);
        }

        if guarantee.report.core_index != core_index as u16 {
            return Err(Error::NotSortedOrUniqueGuarantors);
        }

        // Check if core already has a pending report that hasn't timed out
        if let Some(assignment) = &self.next.avail_assignments[core_index] {
            if slot <= assignment.timeout + 1 {
                return Err(Error::CoreEngaged);
            }
        }
        Ok(())
    }

    fn validate_signatures(&self, guarantee: &ReportGuarantee) -> Result<()> {
        for (_, sig) in guarantee.signatures.iter().enumerate() {
            let validator_index = sig.validator_index as usize;
            if validator_index >= VALIDATORS_COUNT as usize {
                return Err(Error::BadValidatorIndex);
            }
        }

        // Require at least 2/3 guarantors
        if guarantee.signatures.len() < VALIDATORS_SUPER_MAJORITY as usize {
            return Err(Error::InsufficientGuarantees);
        }

        Ok(())
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
