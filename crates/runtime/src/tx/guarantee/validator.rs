//! Reporting validator

use crate::tx::guarantee::error::{Error, Result};
use score::{
    Account, Accounts, CORES_COUNT, EPOCH_LENGTH, Ed25519Public, MAX_DEPENDENCY_COUNT,
    MAX_WORK_REPORT_OUTPUT_SIZE, OpaqueHash, ROTATION_PERIOD, SERVICE_ITEM_MIN_GAS, State,
    TimeSlot, VALIDATORS_COUNT, WORK_REPORT_GAS_LIMIT,
    extrinsic::{GuaranteesExtrinsic, ReportGuarantee},
    params::assignments,
    service::{ReportedWorkPackage, WorkExecResult},
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
    #[tracing::instrument(skip_all, name = "guarantee")]
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
            self.validate_results(guarantee)?;
            self.validate_block(guarantee)?;
            self.validate_deps(guarantee)?;
            let guarantors = self.validate_guarantee(guarantee)?;

            // Record reported package
            reported.push(ReportedWorkPackage {
                hash: guarantee.report.spec.hash,
                exports_root: guarantee.report.spec.exports_root,
            });

            // Record reporters (guarantors)
            reporters.extend(guarantors);
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
        let recent_blocks = self
            .state
            .recent_blocks
            .history
            .iter()
            .map(|b| hex::encode(b.header_hash))
            .collect::<Vec<_>>();

        // GP (11.33)
        let Some(block) = self
            .state
            .recent_blocks
            .history
            .iter()
            .find(|b| b.header_hash == guarantee.report.context.anchor)
        else {
            tracing::warn!(
                "could not find anchor: 0x{} in recent blocks {:?}",
                hex::encode(guarantee.report.context.anchor),
                recent_blocks
            );
            return Err(Error::AnchorNotRecent);
        };

        // GP (11.34)
        if self.timeslot
            < guarantee
                .report
                .context
                .lookup_anchor_slot
                .saturating_sub(score::MAX_AGE_LOOKUP_ANCHOR)
        {
            return Err(Error::FutureReportSlot);
        }

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

    fn validate_guarantee(&mut self, guarantee: &ReportGuarantee) -> Result<Vec<Ed25519Public>> {
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
            .inspect_err(|e| tracing::warn!("failed to get signing message: {:?}", e))
            .map_err(|_| Error::BadSignature)?;
        let mut guarantor = None;
        for sig in guarantee.signatures.iter() {
            let validator_index = sig.validator_index as usize;
            if validator_index >= VALIDATORS_COUNT as usize {
                return Err(Error::BadValidatorIndex);
            }

            if let Some(last) = guarantor
                && validator_index <= last {
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
                .inspect_err(|_| {
                    tracing::warn!(
                        "failed to verify guarantee signature 0x{} by {} - 0x{}",
                        hex::encode(sig.signature),
                        sig.validator_index,
                        hex::encode(key),
                    )
                })
                .map_err(|_| Error::BadSignature)?;
            guarantor = Some(validator_index);
        }

        self.processed.insert(guarantee.report.core_index);

        // Return only the validators who actually provided signatures (reporters)
        Ok(guarantee
            .signatures
            .iter()
            .map(|sig| guarantors[&(sig.validator_index as usize)])
            .collect())
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
                tracing::warn!(
                    "bad code hash for service {}: 0x{} != 0x{}",
                    result.service_id,
                    hex::encode(code_hash),
                    hex::encode(result.code_hash)
                );
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

        let (validators, assignments) = if gslot / ROTATION_PERIOD as u32
            == slot / ROTATION_PERIOD as u32
        {
            let assignments = assignments::core(slot, self.state.entropy[2]);
            (self.state.validators.current, assignments)
        } else {
            let (entropy, validators) = if (slot - ROTATION_PERIOD as u32) / EPOCH_LENGTH
                == slot / EPOCH_LENGTH
            {
                tracing::trace!("last rotation in the same epoch, using current validators");
                (self.state.entropy[2], self.state.validators.current)
            } else {
                tracing::trace!("last rotation in the previous epoch, using previous validators");
                (self.state.entropy[3], self.state.validators.previous)
            };
            let assignments =
                assignments::core(slot.saturating_sub(ROTATION_PERIOD as u32), entropy);
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
