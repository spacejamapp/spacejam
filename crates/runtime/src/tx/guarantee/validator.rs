//! Reporting validator

use crate::tx::guarantee::error::{Error, Result};
use crypto::shuffle;
use score::{
    extrinsic::{GuaranteesExtrinsic, ReportGuarantee},
    safrole::ValidatorIter,
    service::{ReportedWorkPackage, WorkExecResult},
    Account, Accounts, Ed25519Public, OpaqueHash, State, TimeSlot, CORES_COUNT, EPOCH_LENGTH,
    MAX_DEPENDENCY_COUNT, MAX_WORK_REPORT_OUTPUT_SIZE, ROTATION_PERIOD, SERVICE_ITEM_MIN_GAS,
    VALIDATORS_COUNT, WORK_REPORT_GAS_LIMIT,
};
use std::collections::BTreeMap;

/// Context of the reporting module.
pub(super) struct GuaranteeValidator<'s, R: Accounts> {
    pub state: &'s State,
    /// account registry
    pub accounts: &'s R,
    /// core assignments for each validator
    pub core_assignments: Vec<Vec<u16>>,
    /// guarantors for each core
    pub guarantors: BTreeMap<usize, Vec<u16>>,
    /// recent work packages
    pub recent: Vec<ReportedWorkPackage>,
    /// reported work packages
    pub reported: Vec<OpaqueHash>,
    /// The timeslot of the current validation
    pub timeslot: TimeSlot,
}

impl<'s, R: Accounts> GuaranteeValidator<'s, R> {
    /// Create a new reporting validator
    pub fn new(state: &'s State, accounts: &'s R) -> Self {
        Self {
            state,
            accounts,
            core_assignments: vec![],
            guarantors: BTreeMap::new(),
            recent: vec![],
            reported: vec![],
            timeslot: 0,
        }
    }

    /// Validate work reports according to the guarantees extrinsic
    pub fn validate(
        &mut self,
        slot: TimeSlot,
        guarantees: &GuaranteesExtrinsic,
    ) -> Result<(Vec<ReportedWorkPackage>, Vec<Ed25519Public>)> {
        self.init_deps(guarantees);
        self.timeslot = slot;

        // Prepare for reporting
        let mut reported = Vec::new();
        let mut reporters = Vec::new();

        // Process each guarantee
        for guarantee in guarantees.iter() {
            let validators = if self.timeslot / EPOCH_LENGTH == guarantee.slot / EPOCH_LENGTH {
                self.state.validators.current
            } else {
                self.state.validators.previous
            }
            .ed25519();

            self.validate_core(guarantee)?;
            self.validate_rotation(guarantee)?;
            self.validate_results(guarantee)?;
            self.validate_block(guarantee)?;
            self.validate_deps(guarantee)?;
            self.validate_guarantees(guarantee, &validators)?;
            self.validate_guarantors(guarantee)?;

            // Record reported package
            reported.push(ReportedWorkPackage {
                hash: guarantee.report.spec.hash,
                exports_root: guarantee.report.spec.exports_root,
            });

            // Record reporters (guarantors)
            reporters.extend(
                guarantee
                    .signatures
                    .iter()
                    .map(|sig| validators[sig.validator_index as usize]),
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
        let shuffled = shuffle::eq331(
            &(0..VALIDATORS_COUNT as u32)
                .map(|i| (i * CORES_COUNT as u32) / VALIDATORS_COUNT as u32)
                .collect::<Vec<_>>(),
            eta,
        );

        // Calculate rotation offset based on timeslot (P function)
        let rotation = (timeslot % EPOCH_LENGTH) / ROTATION_PERIOD as u32;
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
        let reported = guarantees
            .iter()
            .map(|g| g.report.spec.hash)
            .collect::<Vec<_>>();
        let recent = self
            .state
            .recent_blocks
            .history
            .iter()
            .flat_map(|b| b.reported.clone())
            .collect::<Vec<_>>();

        self.recent = recent;
        self.reported = reported;
    }

    fn validate_block(&self, guarantee: &ReportGuarantee) -> Result<()> {
        let Some(block) = self
            .state
            .recent_blocks
            .history
            .iter()
            .find(|b| b.header_hash == guarantee.report.context.anchor)
        else {
            return Err(Error::AnchorNotRecent);
        };

        // Validate state root
        if block.state_root != guarantee.report.context.state_root {
            return Err(Error::BadStateRoot);
        }

        if block.beefy_root != guarantee.report.context.beefy_root {
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

        let core_index = guarantee.report.core_index as usize;
        if !self.state.pools[core_index].contains(&guarantee.report.authorizer_hash) {
            return Err(Error::CoreUnauthorized);
        }

        Ok(())
    }

    /// Validate work package
    fn validate_deps(&self, guarantee: &ReportGuarantee) -> Result<()> {
        for dep in guarantee.report.context.prerequisites.iter() {
            tracing::debug!("validate_deps: 0x{}", hex::encode(dep));
            if !self.contains_dep(dep) {
                return Err(Error::DependencyMissing);
            }
        }

        if self.duplicated(&guarantee.report.spec.hash) {
            return Err(Error::DuplicatePackage);
        }

        if guarantee.report.context.prerequisites.len() + guarantee.report.lookup.len()
            > MAX_DEPENDENCY_COUNT as usize
        {
            return Err(Error::TooManyDependencies);
        }

        self.validate_segment_lookup(guarantee)?;
        Ok(())
    }

    fn validate_guarantees(
        &self,
        guarantee: &ReportGuarantee,
        validators: &[Ed25519Public],
    ) -> Result<()> {
        let min_guarantees = (VALIDATORS_COUNT as usize / CORES_COUNT) * 2 / 3;
        if guarantee.signatures.len() < min_guarantees {
            return Err(Error::InsufficientGuarantees);
        }

        let message = guarantee.signing_message().map_err(|e| {
            tracing::error!("Error constructing guarantee signing message: {e:?}");
            Error::BadSignature
        })?;

        for sig in guarantee.signatures.iter() {
            let validator_index = sig.validator_index as usize;
            if validator_index >= VALIDATORS_COUNT as usize {
                return Err(Error::BadValidatorIndex);
            }

            // Check if validator is banned before verifying signature
            if self
                .state
                .disputes
                .offenders
                .contains(&validators[validator_index])
            {
                return Err(Error::BannedValidator);
            }

            crypto::ed25519::verify(&message, sig.signature, validators[validator_index]).map_err(
                |e| {
                    tracing::error!("Error verifying ed25519 signature: {e:?}");
                    Error::BadSignature
                },
            )?
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
        tracing::debug!("guarantors: {guarantors:?}");

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
            tracing::error!("core assignment not found");
            return Err(Error::WrongAssignment);
        };

        if guarantors.iter().any(|g| !assignments.contains(g)) {
            tracing::error!("guarantors {guarantors:?} not in assignment {assignments:?}");
            tracing::error!("core assignment: {:?}", self.core_assignments);
            return Err(Error::WrongAssignment);
        }

        self.guarantors.insert(core_index, guarantors);
        Ok(())
    }

    fn validate_results(&self, guarantee: &ReportGuarantee) -> Result<()> {
        let mut output_len = guarantee.report.auth_output.len();
        let mut gas_limit = 0;
        for result in guarantee.report.results.iter() {
            if let WorkExecResult::Ok(blob) = &result.result {
                output_len += blob.len();
                if output_len > MAX_WORK_REPORT_OUTPUT_SIZE {
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

            let Some(code_hash) = self.accounts.code_hash(result.service_id) else {
                return Err(Error::BadServiceId);
            };

            if code_hash != result.code_hash {
                return Err(Error::BadCodeHash);
            }
        }

        Ok(())
    }

    fn validate_rotation(&mut self, guarantee: &ReportGuarantee) -> Result<()> {
        let slot = self.timeslot;
        let gslot = guarantee.slot;
        if gslot > slot {
            return Err(Error::FutureReportSlot);
        }

        if gslot / ROTATION_PERIOD as u32 + 1 < slot / ROTATION_PERIOD as u32 {
            return Err(Error::ReportEpochBeforeLast);
        }

        // TODO: reference GP 11.23
        //
        // The test case or the GP is not correct.
        if gslot / ROTATION_PERIOD as u32 == slot / ROTATION_PERIOD as u32 {
            // if (gslot - ROTATION_PERIOD as u32) / EPOCH_LENGTH == gslot / EPOCH_LENGTH {
            tracing::debug!("core_rotation: current");
            self.assign_cores(slot, self.state.entropy[2]);
            return Ok(());
        } else {
            tracing::debug!("core_rotation: previous");
            self.assign_cores(
                slot.saturating_sub(ROTATION_PERIOD as u32),
                self.state.entropy[3],
            );
        }

        Ok(())
    }

    /// Validate segment lookup
    pub fn validate_segment_lookup(&self, guarantee: &ReportGuarantee) -> Result<()> {
        for (hash, root) in guarantee.report.lookup.iter() {
            if self.reported.contains(hash) {
                continue;
            }

            let Some(reported) = self.recent.iter().find(|r| r.hash == *hash) else {
                return Err(Error::SegmentRootLookupInvalid);
            };

            if reported.exports_root != *root {
                return Err(Error::SegmentRootLookupInvalid);
            }
        }
        Ok(())
    }

    // TODO: check if duplicated in service deps?
    pub fn duplicated(&self, hash: &OpaqueHash) -> bool {
        self.recent.iter().any(|r| r.hash == *hash)
            || self.reported.iter().filter(|h| *h == hash).count() > 1
    }

    fn contains_dep(&self, dep: &OpaqueHash) -> bool {
        self.accounts
            .accounts()
            .iter()
            .any(|(_, a)| a.code() == *dep)
            || self.reported.contains(dep)
            || self.recent.iter().any(|r| r.hash == *dep)
    }
}
