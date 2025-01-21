//! Reporting is the process of reporting the results of a work-package to the service state singleton.

use crypto::shuffle;
use dep::Dependencies;
use score::{
    extrinsic::GuaranteesExtrinsic,
    work::{AvailabilityAssignments, ReportedWorkPackage},
    Ed25519Public, EPOCH_LENGTH, ROTATION_PERIOD,
};
pub use state::{State, StateJson};
use {
    error::{Error, Result},
    score::{
        extrinsic::ReportGuarantee,
        validator::ValidatorData,
        work::{report::WorkExecResult, AvailabilityAssignment},
        OpaqueHash, TimeSlot, CORES_COUNT, MAX_DEPENDENCY_COUNT, MAX_WORK_REPORT_OUTPUT_SIZE,
        SERVICE_ITEM_MIN_GAS, VALIDATORS_COUNT, WORK_REPORT_GAS_LIMIT,
    },
    std::collections::BTreeMap,
};

mod dep;
pub mod error;
mod state;

/// (ρ') Update availability assignments based on guarantees
pub fn reports(
    slot: TimeSlot,
    prev: &AvailabilityAssignments,
    guarantees: &GuaranteesExtrinsic,
) -> Result<AvailabilityAssignments> {
    let mut next = prev.clone();
    for guarantee in guarantees.iter() {
        let core_index = guarantee.report.core_index as usize;
        if core_index >= CORES_COUNT {
            return Err(Error::BadCoreIndex);
        }

        if let Some(Some(assignment)) = prev.get(core_index) {
            if slot <= assignment.timeout + 1 {
                return Err(Error::CoreEngaged);
            }
        }

        next[core_index] = Some(AvailabilityAssignment {
            report: guarantee.report.clone(),
            timeout: slot,
        });
    }

    Ok(next)
}

/// (α') Update authorization pools.
///
/// TODO: check indices
pub fn pools(
    slot: TimeSlot,
    pools: &[Vec<OpaqueHash>; score::CORES_COUNT],
    authorizations: &[Vec<OpaqueHash>; score::CORES_COUNT],
    guarantees: &GuaranteesExtrinsic,
) -> [Vec<OpaqueHash>; score::CORES_COUNT] {
    let mut pools = pools.clone();

    // Process each guarantee
    let mut processed = Vec::new();
    for guarantee in guarantees {
        // Consume the authorizer from the pool
        pools[guarantee.report.core_index as usize] = pools[guarantee.report.core_index as usize]
            .iter()
            .filter(|pool| **pool != guarantee.report.authorizer_hash)
            .cloned()
            .collect();

        // mark the core as processed
        processed.push(guarantee.report.core_index as usize);
    }

    // add new authorizers from queue to the pools
    for (core_index, pool) in pools.iter_mut().enumerate() {
        if !processed.contains(&core_index) {
            *pool = pool[1..].into();
        }

        pool.push(authorizations[core_index][slot as usize]);
    }

    pools
}

/// Report the work packages
///
/// TODO: refactor the state on connecting storage.
pub fn report(
    state: &score::State,
    slot: TimeSlot,
    guarantees: &GuaranteesExtrinsic,
) -> Result<(Vec<ReportedWorkPackage>, Vec<Ed25519Public>)> {
    let pstate = State::from(state.clone());
    let mut context = Context::from(&pstate);
    context.validate(slot, guarantees)
}

/// Context of the reporting module.
pub struct Context<'s> {
    pub state: &'s State,
    pub validators: Vec<ValidatorData>,
    pub deps: Dependencies,
    pub core_assignments: Vec<Vec<u16>>,
    pub guarantors: BTreeMap<usize, Vec<u16>>,
}

impl Context<'_> {
    /// Validate work reports according to the guarantees extrinsic
    pub fn validate(
        &mut self,
        slot: TimeSlot,
        guarantees: &GuaranteesExtrinsic,
    ) -> Result<(Vec<ReportedWorkPackage>, Vec<Ed25519Public>)> {
        self.init_deps(guarantees);

        // Prepare for reporting
        let mut reported = Vec::new();
        let mut reporters = Vec::new();
        let (service_ids, code_hashes): (Vec<_>, Vec<_>) = self
            .state
            .services
            .iter()
            .map(|s| (s.id, s.info.code))
            .unzip();

        // Process each guarantee
        for guarantee in guarantees.iter() {
            self.validate_core(guarantee)?;
            self.validate_rotation(slot, guarantee)?;
            self.validate_results(&code_hashes, &service_ids, guarantee)?;
            self.validate_block(guarantee)?;
            self.validate_deps(guarantee)?;
            self.validate_guarantees(guarantee)?;
            self.validate_guarantors(guarantee)?;

            // Record reported package
            reported.push(ReportedWorkPackage {
                hash: guarantee.report.package_spec.hash,
                exports_root: guarantee.report.package_spec.exports_root,
            });

            // Record reporters (guarantors)
            reporters.extend(
                guarantee
                    .signatures
                    .iter()
                    .map(|sig| self.validators[sig.validator_index as usize].ed25519),
            );
        }

        // FIXME: not sure if we need to sort the reporters here since it's not related to
        // storage directly, there was a similar problem in the `dispute` module.
        reporters.sort();
        reported.sort_by(|a, b| a.hash.cmp(&b.hash));
        Ok((reported, reporters))
    }

    /// Assign cores to validators based on the timeslot
    fn assign_cores(&mut self, timeslot: u32, eta: [u8; 32]) {
        let initial_sequence: Vec<u32> = (0..VALIDATORS_COUNT as u32)
            .map(|i| (i * CORES_COUNT as u32) / VALIDATORS_COUNT as u32)
            .collect();

        // Calculate rotation offset based on timeslot
        let rotation = (timeslot % EPOCH_LENGTH) / ROTATION_PERIOD;

        // First shuffle using epoch entropy
        let shuffled = shuffle::eq331(&initial_sequence, eta);

        // Apply rotation to the shuffled sequence
        let rotated: Vec<u32> = shuffled
            .iter()
            .map(|&core_idx| (core_idx + rotation) % CORES_COUNT as u32)
            .collect();

        // Group validators by their assigned cores
        let mut assignments: Vec<Vec<u16>> = vec![Vec::new(); CORES_COUNT];
        for (validator_idx, &core_idx) in rotated.iter().enumerate() {
            assignments[core_idx as usize].push(validator_idx as u16);
        }

        self.core_assignments = assignments;
    }

    fn init_deps(&mut self, guarantees: &GuaranteesExtrinsic) {
        let service = self
            .state
            .services
            .iter()
            .map(|s| s.info.code)
            .collect::<Vec<_>>();
        let reported = guarantees
            .iter()
            .map(|g| g.report.package_spec.hash)
            .collect::<Vec<_>>();
        let recent = self
            .state
            .recent_blocks
            .iter()
            .flat_map(|b| b.reported.clone())
            .collect::<Vec<_>>();

        self.deps = Dependencies {
            service,
            recent,
            reported,
        };
    }

    fn validate_block(&self, guarantee: &ReportGuarantee) -> Result<()> {
        let Some(block) = self
            .state
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

        if block.mmr.root() != Some(guarantee.report.context.beefy_root) {
            return Err(Error::BadBeefyMmrRoot);
        }

        Ok(())
    }

    fn validate_core(&self, guarantee: &ReportGuarantee) -> Result<()> {
        // NOTE: This has already been checked in the [reports] function.
        //
        // if guarantee.report.core_index >= CORES_COUNT as u16 {
        //     return Err(Error::BadCoreIndex);
        // }

        if !self.state.auth_pools[guarantee.report.core_index as usize]
            .contains(&guarantee.report.authorizer_hash)
        {
            return Err(Error::CoreUnauthorized);
        }

        Ok(())
    }

    /// Validate work package
    fn validate_deps(&self, guarantee: &ReportGuarantee) -> Result<()> {
        for dep in guarantee.report.context.prerequisites.iter() {
            if !self.deps.contains(dep) {
                return Err(Error::DependencyMissing);
            }
        }

        if self.deps.duplicated(&guarantee.report.package_spec.hash) {
            return Err(Error::DuplicatePackage);
        }

        if guarantee.report.context.prerequisites.len() + guarantee.report.reported.len()
            > MAX_DEPENDENCY_COUNT
        {
            return Err(Error::TooManyDependencies);
        }

        self.deps.validate_segment_lookup(guarantee)?;
        Ok(())
    }

    fn validate_guarantees(&self, guarantee: &ReportGuarantee) -> Result<()> {
        let min_guarantees = (VALIDATORS_COUNT as usize / CORES_COUNT) * 2 / 3;
        if guarantee.signatures.len() < min_guarantees {
            return Err(Error::InsufficientGuarantees);
        }

        let message = guarantee
            .signing_message()
            .map_err(|_| Error::BadSignature)?;

        for sig in guarantee.signatures.iter() {
            let validator_index = sig.validator_index as usize;
            if validator_index >= VALIDATORS_COUNT as usize {
                return Err(Error::BadValidatorIndex);
            }

            crypto::ed25519::verify(
                &message,
                sig.signature,
                self.validators[validator_index].ed25519,
            )
            .map_err(|_| Error::BadSignature)?
        }

        Ok(())
    }

    fn validate_guarantors(&mut self, guarantee: &ReportGuarantee) -> Result<()> {
        let core_index = guarantee.report.core_index as usize;
        let guarantors = guarantee
            .signatures
            .iter()
            .map(|sig| sig.validator_index)
            .collect::<Vec<_>>();

        let guaranteed = self.guarantors.values().flatten().collect::<Vec<_>>();
        if guarantors.iter().any(|g| guaranteed.contains(&g)) {
            return Err(Error::OutOfOrderGuarantee);
        }

        if guarantee
            .signatures
            .windows(2)
            .any(|w| w[0].validator_index > w[1].validator_index)
        {
            return Err(Error::NotSortedOrUniqueGuarantors);
        }

        let Some(assignments) = self.core_assignments.get(core_index) else {
            return Err(Error::WrongAssignment);
        };

        if guarantors.iter().any(|g| !assignments.contains(g)) {
            return Err(Error::WrongAssignment);
        }

        self.guarantors.insert(core_index, guarantors);
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

    fn validate_rotation(&mut self, slot: TimeSlot, guarantee: &ReportGuarantee) -> Result<()> {
        if guarantee.slot > slot {
            return Err(Error::FutureReportSlot);
        }

        // TODO: reference GP 11.23
        //
        // The test case or the GP is not correct.
        if guarantee.slot / ROTATION_PERIOD == slot / ROTATION_PERIOD {
            self.validators = self.state.curr_validators.clone();
            self.assign_cores(slot, self.state.entropy[2]);
            return Ok(());
        } else {
            self.validators = self.state.prev_validators.clone();
            self.assign_cores(slot.saturating_sub(ROTATION_PERIOD), self.state.entropy[3]);
        }

        if guarantee.slot / ROTATION_PERIOD + 1 < slot / ROTATION_PERIOD {
            return Err(Error::ReportEpochBeforeLast);
        }

        Ok(())
    }
}

impl<'s> From<&'s State> for Context<'s> {
    fn from(state: &'s State) -> Self {
        Self {
            validators: state.curr_validators.clone(),
            state,
            core_assignments: vec![],
            guarantors: BTreeMap::new(),
            deps: Dependencies::default(),
        }
    }
}
