//! Reporting is the process of reporting the results of a work-package to the service state singleton.

use {
    error::{Error, Result},
    score::{
        extrinsic::ReportGuarantee,
        misc::{OpaqueHash, TimeSlot},
        work::{report::WorkExecResult, AvailabilityAssignment},
        CORES_COUNT, MAX_DEPENDENCY_COUNT, MAX_WORK_REPORT_OUTPUT_SIZE, SERVICE_ITEM_MIN_GAS,
        VALIDATORS_COUNT, WORK_REPORT_GAS_LIMIT,
    },
    state::{Input, Output, ReportedPackage, State},
};

pub mod error;
pub mod state;

/// Handler of the reporting module.
pub struct Handler {
    pub prev: State,
    pub next: State,
    pub deps: Dependencies,
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

        self.init_deps(&input);

        // Process each guarantee
        for (core_index, guarantee) in input.guarantees.into_iter().enumerate() {
            self.validate_core(input.slot, core_index, &guarantee)?;
            self.validate_results(&code_hashes, &service_ids, &guarantee)?;
            self.validate_block(&guarantee)?;
            self.validate_signatures(&guarantee)?;
            self.validate_package(&guarantee)?;

            // Record reported package
            reported.push(ReportedPackage {
                work_package_hash: guarantee.report.package_spec.hash,
                segment_tree_root: guarantee.report.package_spec.exports_root,
            });

            // Create availability assignment
            let assignment = AvailabilityAssignment {
                report: guarantee.report,
                timeout: input.slot,
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

        // FIXME: not sure if we need to sort the reporters here since it's not related to
        // storage directly, there was a similar problem in the `dispute` module.
        reporters.sort();
        Ok(Output {
            reported,
            reporters,
        })
    }

    fn init_deps(&mut self, input: &Input) {
        let service = self
            .prev
            .services
            .iter()
            .map(|s| s.info.code_hash)
            .collect::<Vec<_>>();
        let reported = input
            .guarantees
            .iter()
            .map(|g| g.report.package_spec.hash)
            .collect::<Vec<_>>();
        let recent = self
            .prev
            .recent_blocks
            .iter()
            .map(|b| b.reported.iter().map(|r| r.hash).collect::<Vec<_>>())
            .flatten()
            .collect::<Vec<_>>();

        self.deps = Dependencies {
            service,
            recent,
            reported,
        };
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
        if !self.next.auth_pools[core_index].contains(&guarantee.report.authorizer_hash) {
            return Err(Error::CoreUnauthorized);
        }

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

    /// Validate work package
    fn validate_package(&mut self, guarantee: &ReportGuarantee) -> Result<()> {
        for dep in guarantee.report.context.prerequisites.iter() {
            if !self.deps.contains(dep) {
                self.next = self.prev.clone();
                return Err(Error::DependencyMissing);
            }
        }

        if self.deps.duplicated(&guarantee.report.package_spec.hash) {
            return Err(Error::DuplicatePackage);
        }

        if guarantee.report.context.prerequisites.len() + guarantee.report.segment_root_lookup.len()
            > MAX_DEPENDENCY_COUNT
        {
            return Err(Error::TooManyDependencies);
        }

        Ok(())
    }

    fn validate_results(
        &self,
        code_hashes: &[OpaqueHash],
        service_ids: &[u32],
        guarantee: &ReportGuarantee,
    ) -> Result<()> {
        let mut output_len = guarantee.report.auth_output.len();
        let mut gas_limit = 0;
        for result in guarantee.report.results.iter() {
            if let WorkExecResult::Ok(blob) = &result.result {
                output_len += blob.len();
                if output_len >= MAX_WORK_REPORT_OUTPUT_SIZE {
                    return Err(Error::WorkReportTooBig);
                }
            }

            gas_limit += result.accumulate_gas;
            if gas_limit > WORK_REPORT_GAS_LIMIT {
                return Err(Error::WorkReportGasTooHigh);
            }

            if result.accumulate_gas < SERVICE_ITEM_MIN_GAS {
                return Err(Error::ServiceItemGasTooLow);
            }

            if !code_hashes.contains(&result.code_hash) {
                return Err(Error::BadCodeHash);
            }

            if !service_ids.contains(&result.service_id) {
                return Err(Error::BadServiceId);
            }
        }

        Ok(())
    }

    fn validate_signatures(&self, guarantee: &ReportGuarantee) -> Result<()> {
        let message = guarantee
            .signing_message()
            .map_err(|_| Error::BadSignature)?;
        for (_, sig) in guarantee.signatures.iter().enumerate() {
            let validator_index = sig.validator_index as usize;
            if validator_index >= VALIDATORS_COUNT as usize {
                return Err(Error::BadValidatorIndex);
            }

            if let Err(_) = crypto::ed25519::verify(
                &message,
                sig.signature,
                self.next.curr_validators[validator_index].ed25519,
            ) {
                return Err(Error::BadSignature);
            }
        }

        // Require at least 2/3 guarantors
        //
        // FIXME: this is not correct, we need to check the number of guarantors
        /* if guarantee.signatures.len() < VALIDATORS_SUPER_MAJORITY as usize {
            return Err(Error::InsufficientGuarantees);
        } */

        Ok(())
    }
}

impl From<State> for Handler {
    fn from(state: State) -> Self {
        Self {
            prev: state.clone(),
            next: state,
            deps: Dependencies::default(),
        }
    }
}

/// Temp dependencies for validation
#[derive(Default)]
pub struct Dependencies {
    pub service: Vec<OpaqueHash>,
    pub recent: Vec<OpaqueHash>,
    pub reported: Vec<OpaqueHash>,
}

impl Dependencies {
    fn contains(&self, hash: &OpaqueHash) -> bool {
        self.service.contains(hash) || self.recent.contains(hash) || self.reported.contains(hash)
    }

    // TODO: duplicated in service deps?
    fn duplicated(&self, hash: &OpaqueHash) -> bool {
        self.recent.contains(hash) || self.reported.iter().filter(|h| *h == hash).count() > 1
    }
}
