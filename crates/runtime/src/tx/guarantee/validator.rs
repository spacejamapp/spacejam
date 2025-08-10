//! Reporting validator

use crate::tx::guarantee::error::{Error, Result};
use score::{
    extrinsic::{GuaranteesExtrinsic, ReportGuarantee},
    safrole::ValidatorIter,
    service::{ReportedWorkPackage, WorkExecResult},
    util, Account, Accounts, Ed25519Public, OpaqueHash, State, TimeSlot, CORES_COUNT, EPOCH_LENGTH,
    MAX_DEPENDENCY_COUNT, MAX_WORK_REPORT_OUTPUT_SIZE, ROTATION_PERIOD, SERVICE_ITEM_MIN_GAS,
    VALIDATORS_COUNT, WORK_REPORT_GAS_LIMIT,
};
use std::collections::{BTreeMap, BTreeSet};

/// Context of the reporting module.
pub(super) struct GuaranteeValidator<'s, R: Accounts> {
    pub state: &'s State,
    /// account registry
    pub accounts: &'s R,
    /// guarantors for each core
    pub processed: BTreeSet<u16>,
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
            processed: BTreeSet::new(),
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
        let mut reporters = BTreeSet::new();

        // Process each guarantee
        for guarantee in guarantees.iter() {
            let validators = if self.timeslot / EPOCH_LENGTH == guarantee.slot / EPOCH_LENGTH {
                self.state.validators.current
            } else {
                self.state.validators.previous
            }
            .ed25519();

            self.validate_results(guarantee)?;
            self.validate_block(guarantee)?;
            self.validate_deps(guarantee)?;
            self.validate_guarantee(guarantee)?;

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

        // Sort the reported work packages and reporters
        reported.sort_by(|a, b| a.hash.cmp(&b.hash));
        Ok((reported, reporters.into_iter().collect()))
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

    /// Validate work package
    fn validate_deps(&self, guarantee: &ReportGuarantee) -> Result<()> {
        if !self.state.pools[guarantee.report.core_index as usize]
            .contains(&guarantee.report.authorizer_hash)
        {
            return Err(Error::CoreUnauthorized);
        }

        for dep in guarantee.report.context.prerequisites.iter() {
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

    fn validate_guarantee(&mut self, guarantee: &ReportGuarantee) -> Result<()> {
        // 1. validate the rotation
        let guarantors = self.validate_rotation(guarantee)?;

        // 2. Check if the core has been processed
        if self.processed.contains(&guarantee.report.core_index) {
            return Err(Error::OutOfOrderGuarantee);
        }

        // 3. validate the number of guarantees
        let min_guarantees = (VALIDATORS_COUNT as usize / CORES_COUNT) * 2 / 3;
        if guarantee.signatures.len() < min_guarantees {
            return Err(Error::InsufficientGuarantees);
        }

        // 4. validate the signatures
        let message = guarantee
            .signing_message()
            .map_err(|_| Error::BadSignature)?;
        let mut guarantor = 0;
        for sig in guarantee.signatures.iter() {
            let validator_index = sig.validator_index as usize;
            if validator_index >= VALIDATORS_COUNT as usize {
                return Err(Error::BadValidatorIndex);
            }

            if validator_index < guarantor {
                return Err(Error::NotSortedOrUniqueGuarantors);
            }

            let Some(key) = guarantors.get(&validator_index) else {
                return Err(Error::WrongAssignment);
            };

            // Check if validator is banned before verifying signature
            if self.state.disputes.offenders.contains(key) {
                return Err(Error::BannedValidator);
            }

            crypto::ed25519::verify(&message, sig.signature, *key)
                .map_err(|_| Error::BadSignature)?;

            guarantor = validator_index;
        }

        self.processed.insert(guarantee.report.core_index);
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

    fn validate_rotation(
        &mut self,
        guarantee: &ReportGuarantee,
    ) -> Result<BTreeMap<usize, Ed25519Public>> {
        let slot = self.timeslot;
        let gslot = guarantee.slot;
        if gslot > slot {
            return Err(Error::FutureReportSlot);
        }

        if gslot / ROTATION_PERIOD as u32 + 1 < slot / ROTATION_PERIOD as u32 {
            return Err(Error::ReportEpochBeforeLast);
        }

        // GP reference: (11.26)
        let (validators, assignments) =
            if gslot / ROTATION_PERIOD as u32 == slot / ROTATION_PERIOD as u32 {
                let assignments = util::assignments(slot, self.state.entropy[2]);
                (self.state.validators.current, assignments)
            } else {
                let (entropy, validators) =
                    if (slot - ROTATION_PERIOD as u32) / EPOCH_LENGTH == slot / EPOCH_LENGTH {
                        (self.state.entropy[2], self.state.validators.current)
                    } else {
                        (self.state.entropy[3], self.state.validators.previous)
                    };
                let assignments =
                    util::assignments(slot.saturating_sub(ROTATION_PERIOD as u32), entropy);
                (validators, assignments)
            };

        // Get the guarantors for the core
        Ok(assignments[guarantee.report.core_index as usize]
            .iter()
            .map(|v| (*v as usize, validators[*v as usize].ed25519))
            .collect())
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
