//! Reporting validator

use crate::tx::guarantee::error::{Error, Result};
use account::{Account, Accounts};
use crypto::ed25519::SigItem;
use score::{
    CORES_COUNT, CoreIndex, EPOCH_LENGTH, Ed25519Public, Entropy, MAX_DEPENDENCY_COUNT,
    MAX_WORK_REPORT_OUTPUT_SIZE, OpaqueHash, ROTATION_PERIOD, State, TimeSlot, VALIDATORS_COUNT,
    WORK_REPORT_GAS_LIMIT,
    extrinsic::{GuaranteesExtrinsic, ReportGuarantee},
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

    /// Validate work reports according to the guarantees extrinsic.
    #[tracing::instrument(skip_all, name = "guarantee")]
    pub fn validate(
        &mut self,
        slot: TimeSlot,
        guarantees: &GuaranteesExtrinsic,
    ) -> Result<(Vec<ReportedWorkPackage>, Vec<Ed25519Public>, Vec<SigItem>)> {
        self.init_deps(guarantees);
        self.timeslot = slot;

        // Prepare for reporting
        let mut reported = Vec::new();
        let mut reporters = BTreeSet::new();
        let mut triples = Vec::new();

        // Process each guarantee
        for guarantee in guarantees.iter() {
            self.validate_results(guarantee)?;
            self.validate_block(guarantee)?;
            self.validate_deps(guarantee)?;
            let (guarantors, mut g_triples) = self.validate_guarantee(guarantee)?;

            // Record reported package
            reported.push(ReportedWorkPackage {
                hash: guarantee.report.spec.hash,
                exports_root: guarantee.report.spec.exports_root,
            });

            // Record reporters (guarantors)
            reporters.extend(guarantors);
            triples.append(&mut g_triples);
        }

        // Sort the reported work packages and reporters
        reported.sort_by_key(|a| a.hash);
        Ok((reported, reporters.into_iter().collect(), triples))
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
            .flat_map(|b| b.reported.iter().cloned())
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
            tracing::debug!(
                "could not find anchor: 0x{} in recent blocks {:?}",
                hex::encode(guarantee.report.context.anchor),
                recent_blocks
            );
            return Err(Error::AnchorNotRecent);
        };

        // GP (11.34): lookup anchor slot must be within the recent window
        if guarantee.report.context.lookup_anchor_slot
            < self.timeslot.saturating_sub(score::MAX_AGE_LOOKUP_ANCHOR)
        {
            return Err(Error::LookupAnchorNotRecent);
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
        // FIXME:
        //
        // This is checked in the reports function as well.
        if guarantee.report.core_index as usize >= CORES_COUNT {
            return Err(Error::BadCoreIndex);
        }

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

    fn validate_guarantee(
        &mut self,
        guarantee: &ReportGuarantee,
    ) -> Result<(Vec<Ed25519Public>, Vec<SigItem>)> {
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

        // 4. Semantic checks; collect triples for the caller to batch-verify.
        let message = guarantee.signing_message();
        let mut guarantor = None;
        let mut triples = Vec::with_capacity(guarantee.signatures.len());
        for sig in guarantee.signatures.iter() {
            let validator_index = sig.validator_index as usize;
            if validator_index >= VALIDATORS_COUNT as usize {
                return Err(Error::BadValidatorIndex);
            }

            if let Some(last) = guarantor
                && validator_index <= last
            {
                return Err(Error::NotSortedOrUniqueGuarantors);
            }

            let Some(key) = guarantors.get(&validator_index) else {
                return Err(Error::WrongAssignment);
            };

            // Check if validator is banned before verifying signature
            if self.state.disputes.offenders.contains(key) {
                return Err(Error::BannedValidator);
            }

            triples.push(SigItem {
                message: message.clone(),
                signature: sig.signature,
                key: *key,
            });
            guarantor = Some(validator_index);
        }

        self.processed.insert(guarantee.report.core_index);

        // Return only the validators who actually provided signatures (reporters)
        let reporters = guarantee
            .signatures
            .iter()
            .map(|sig| guarantors[&(sig.validator_index as usize)])
            .collect();
        Ok((reporters, triples))
    }

    fn validate_results(&self, guarantee: &ReportGuarantee) -> Result<()> {
        let mut output_len = guarantee.report.auth_output.len();
        let mut gas_limit: u64 = 0;
        for result in guarantee.report.results.iter() {
            if let WorkExecResult::Ok(blob) = &result.result {
                output_len = output_len.wrapping_add(blob.len());
                if output_len > MAX_WORK_REPORT_OUTPUT_SIZE {
                    return Err(Error::WorkReportTooBig);
                }
            }

            // wrapping (not saturating) to match polkajam's modular u64 sum.
            gas_limit = gas_limit.wrapping_add(result.accumulate_gas);
            if gas_limit > WORK_REPORT_GAS_LIMIT {
                return Err(Error::WorkReportGasTooHigh);
            }

            let Some(code_hash) = self.accounts.code_hash(result.service_id) else {
                return Err(Error::BadServiceId);
            };

            let min_gas = self
                .accounts
                .min_acc_gas(result.service_id)
                .ok_or(Error::BadServiceId)?;
            if result.accumulate_gas < min_gas {
                return Err(Error::ServiceItemGasTooLow);
            }

            if code_hash != result.code_hash {
                tracing::debug!(
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
        &self,
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

        // get the validators and assignments
        let (validators, assignments) = if gslot / ROTATION_PERIOD as u32
            == slot / ROTATION_PERIOD as u32
        {
            let assignments = self::permute(self.state.entropy[2], slot);
            (&self.state.validators.current, assignments)
        } else {
            let (entropy, validators) =
                if (slot - ROTATION_PERIOD as u32) / EPOCH_LENGTH == slot / EPOCH_LENGTH {
                    (self.state.entropy[2], &self.state.validators.current)
                } else {
                    (self.state.entropy[3], &self.state.validators.previous)
                };
            let assignments = self::permute(entropy, slot.saturating_sub(ROTATION_PERIOD as u32));
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

    pub fn duplicated(&self, hash: &OpaqueHash) -> bool {
        self.reported.iter().filter(|h| *h == hash).count() > 1
            || self.recent.iter().any(|r| r.hash == *hash)
            || self.state.history.iter().flatten().any(|h| h == hash)
            || self
                .state
                .queue
                .iter()
                .flatten()
                .any(|r| r.report.spec.hash == *hash)
            || self
                .state
                .reports
                .iter()
                .flatten()
                .any(|a| a.report.spec.hash == *hash)
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

/// Permute function P(e, t) for guarantor assignments.
///
/// Returns the core assignments for all validators based on entropy and time.
fn permute(entropy: Entropy, timeslot: TimeSlot) -> score::Array<Vec<CoreIndex>, CORES_COUNT> {
    let initial_assignments: Vec<u32> = (0..VALIDATORS_COUNT as u32)
        .map(|i| (CORES_COUNT as u32 * i) / VALIDATORS_COUNT as u32)
        .collect();
    let shuffled = crypto::shuffle::eq331(&initial_assignments, entropy);

    // Apply rotation and convert to CoreIndex
    let rotation_offset = (timeslot % EPOCH_LENGTH) / ROTATION_PERIOD as u32;
    self::rotate(
        shuffled.into_iter().map(|x| x as CoreIndex).collect(),
        rotation_offset,
    )
}

/// Rotation function R for guarantor assignments.
///
/// Rotates core assignments by n positions.
fn rotate(assignments: Vec<CoreIndex>, n: u32) -> score::Array<Vec<CoreIndex>, CORES_COUNT> {
    let rotated: Vec<CoreIndex> = assignments
        .iter()
        .map(|&x| ((x as u32 + n) % CORES_COUNT as u32) as CoreIndex)
        .collect();

    // Group validators by their assigned cores
    let mut assignments: score::Array<Vec<u16>, CORES_COUNT> = Default::default();
    for (validator_idx, &core_idx) in rotated.iter().enumerate() {
        assignments[core_idx as usize].push(validator_idx as u16);
    }

    assignments
}
