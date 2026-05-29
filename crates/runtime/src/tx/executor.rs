//! Block state-transition executor.
//!
//! Wraps the four-round STF pipeline into a single struct with one method per
//! round. Guarantee/assurance ed25519 batch verify runs in parallel with the
//! ticket::safrole ring-VRF via `rayon::join`.

use crate::{
    Storage,
    account::Accounts,
    storage::Commit,
    tx::{assurance, block, dispute, guarantee, preimage, ticket},
};
use account::Accounts as _;
use anyhow::Result;
use pvm::Pvm;
use score::{
    Block, CORES_COUNT, EPOCH_LENGTH, Ed25519Public, OpaqueHash, State, TrieKey,
    block::header::{EpochMark, TicketsMark},
    extrinsic::dispute::DisputesRecords,
    safrole::{Safrole, ValidatorIter},
    service::{AvailabilityAssignments, ReportedWorkPackage, WorkReport},
};
use std::{marker::PhantomData, sync::Arc, thread};

/// Four-round block state-transition executor.
pub struct Executor<'a, Vm: Pvm, S: Storage> {
    block: &'a mut Block,
    state: State,
    accounts: Option<Accounts<S>>,
    new_epoch: bool,

    // round-to-round handoff
    dispute_records: DisputesRecords,
    reports: AvailabilityAssignments,
    reported: Vec<ReportedWorkPackage>,
    reporters: Vec<Ed25519Public>,
    available: Vec<WorkReport>,
    counts: [u32; CORES_COUNT],
    root: OpaqueHash,
    _vm: PhantomData<Vm>,
}

impl<'a, Vm: Pvm, S: Storage> Executor<'a, Vm, S> {
    /// Initialize the executor for a block + prior state.
    pub fn new(block: &'a mut Block, state: State, storage: Arc<S>) -> Self {
        let new_epoch = block.header.epoch() > (state.timeslot / EPOCH_LENGTH);
        Self {
            block,
            state,
            accounts: Some(Accounts::new(storage)),
            new_epoch,
            dispute_records: DisputesRecords::default(),
            reports: AvailabilityAssignments::default(),
            reported: vec![],
            reporters: vec![],
            available: vec![],
            counts: [0u32; CORES_COUNT],
            root: [0u8; 32],
            _vm: PhantomData,
        }
    }

    /// Run the four-round STF and emit the resulting state diff.
    #[tracing::instrument(skip_all, name = "stf")]
    pub fn run(mut self) -> Result<Commit<TrieKey, Vec<u8>>> {
        self.validate_extrinsics()?;
        self.update_reports()?;
        self.accumulate()?;
        self.finalize()
    }

    /// Round 1 — validate the block against prior state.
    ///
    /// - extrinsic hash check
    /// - preimages (E_P) (12.6)
    /// - entropy update (η') (6.22)
    /// - disputes (ψ') (10.4)
    /// - validator rotation (λ', κ') on epoch change (6.13)
    /// - last-block state-root patch
    /// - `rayon::join`: guarantee/assurance ed25519 batch ∥ ticket::safrole
    ///   ring-VRF (γ') (12.10)
    fn validate_extrinsics(&mut self) -> Result<()> {
        if self.block.extrinsic.hash() != self.block.header.extrinsic_hash {
            anyhow::bail!("extrinsic hash mismatch");
        }

        // (E_P) Validate preimages against prior state (12.6)
        let accounts = self.accounts.as_mut().expect("accounts present");
        preimage::validate(accounts, &self.block.extrinsic.preimages)?;

        // (η') Update entropy (6.22)
        let entropy =
            crypto::vrf::ietf_output(self.block.header.entropy_source).unwrap_or_default();
        self.state.entropy = ticket::eta(self.new_epoch, &self.state.entropy, entropy);

        // (ψ') Update disputes against prior validator sets (10.4)
        self.dispute_records = if self.block.extrinsic.disputes.is_empty() {
            if !self.block.header.offenders_mark.is_empty() {
                anyhow::bail!("offenders mark is not empty");
            }
            DisputesRecords::default()
        } else {
            let (next_psi, records, triples) = dispute::disputes(
                self.state.timeslot,
                &self.state.validators.current,
                &self.state.validators.previous,
                std::mem::take(&mut self.state.disputes),
                &self.block.extrinsic.disputes,
            )?;
            crypto::ed25519::SigItem::batch_verify(&triples)?;
            self.state.disputes = next_psi;
            self.block.header.offenders_mark = records.offenders.clone();
            records
        };

        // (λ', κ') Update validator state on epoch change (6.13)
        if self.new_epoch {
            self.state.validators.previous = std::mem::replace(
                &mut self.state.validators.current,
                self.state.safrole.validators.clone(),
            );
        }

        // Patch the parent-state-root field on the last block of history
        if let Some(last) = self.state.recent_blocks.history.last_mut() {
            last.state_root = self.block.header.parent_state_root;
        }

        self.sigs_and_safrole_parallel()
    }

    /// Round 2 — apply availability and guarantee outcomes to report assignments.
    ///
    /// - availability outcomes (ρ‡) (11.17)
    /// - new guarantees (ρ') (11.43)
    fn update_reports(&mut self) -> Result<()> {
        let reports = std::mem::take(&mut self.reports);
        let reports = assurance::reports(self.block.header.slot, &self.available, reports);
        self.state.reports = guarantee::reports(
            self.block.header.slot,
            &reports,
            &self.block.extrinsic.guarantees,
        )?;
        Ok(())
    }

    /// Round 3 — statistics and accumulation.
    ///
    /// - statistics update (π')
    /// - accumulate available work reports via the PVM
    /// - merge accumulation result into state fields
    /// - spawn `ticket::lazy::drawn` warmer for the next safrole candidate
    fn accumulate(&mut self) -> Result<()> {
        self.state.statistics.update(
            self.new_epoch,
            self.block.header.author_index,
            &self.block.extrinsic,
        )?;
        self.state
            .statistics
            .merge_reports(&self.available, &self.counts);

        let available = std::mem::take(&mut self.available);
        let accounts = self.accounts.take().expect("accounts present");

        let accumulation = guarantee::accumulate::<Vm, _>(
            self.block.header.slot,
            self.state.timeslot,
            available,
            &self.state.queue,
            &self.state.history,
            &self.state.privileges,
            &self.state.validators.drawn,
            &self.state.authorization,
            accounts,
            self.state.entropy,
        )?;

        self.state.privileges = accumulation.privileges;
        self.state.queue = accumulation.ready_queue;
        self.state.history = accumulation.accumulated_queue;
        self.state.validators.drawn = accumulation.validators;
        self.state.authorization = accumulation.authorization;

        let candidate = self
            .state
            .safrole
            .next(&self.state.validators.drawn, &self.state.disputes.offenders);
        thread::spawn(move || ticket::lazy::drawn(&candidate));

        self.state.statistics.merge_services(accumulation.records);
        self.state.logs = accumulation.logs;
        self.root = accumulation.root;
        self.accounts = Some(accumulation.accounts);
        Ok(())
    }

    /// Round 4 — commit block and emit the state diff.
    ///
    /// - block history (β')
    /// - reporter statistics merge
    /// - preimage integration (δ')
    /// - authorization pools (α') (12.13)
    /// - timeslot (τ')
    /// - flush state pairs into the final diff
    fn finalize(mut self) -> Result<Commit<TrieKey, Vec<u8>>> {
        let mut diff = Commit::default();

        // (β') Update the block history
        block::history::import(
            &mut self.state.recent_blocks,
            self.block.header.hash(),
            self.root,
            std::mem::take(&mut self.reported),
        );

        if !self.reporters.is_empty() {
            self.state
                .statistics
                .merge_reporters(&self.reporters, &self.state.validators.current.ed25519())?;
        }

        // (δ') Integrate preimages into the post-transfer state
        let accounts = self.accounts.take().expect("accounts present");
        let accounts = preimage::accounts(
            self.block.header.slot,
            std::mem::take(&mut self.block.extrinsic.preimages),
            accounts,
        );
        let (updates, removals) = accounts.diff();
        diff.extend_iter(updates, removals);

        // (α') Update the authorization pools (12.13)
        self.state.pools = guarantee::pools(
            self.block.header.slot,
            &self.state.pools,
            &self.state.authorization,
            &self.block.extrinsic.guarantees,
        );

        // (τ') Update the timeslot
        self.state.timeslot = self.block.header.slot;

        diff.update
            .extend(self.state.pairs(self.new_epoch, &self.block.extrinsic));
        Ok(diff)
    }

    /// Run guarantee/assurance sig collect + batch_verify in parallel with
    /// ticket::safrole ring-VRF.
    fn sigs_and_safrole_parallel(&mut self) -> Result<()> {
        let new_epoch = self.new_epoch;
        let needs_safrole = !self.block.extrinsic.tickets.is_empty() || new_epoch;
        let safrole_in = needs_safrole.then(|| std::mem::take(&mut self.state.safrole));
        let state_view: &State = &self.state;
        let block_view: &Block = &*self.block;
        let accounts_view = self.accounts.as_ref().expect("accounts present");
        let dispute_records_view = &self.dispute_records;
        let (sigs_res, safrole_res) = rayon::join(
            || {
                Self::sigs_branch(
                    state_view,
                    accounts_view,
                    block_view,
                    dispute_records_view,
                    new_epoch,
                )
            },
            || {
                safrole_in
                    .map(|s| Self::safrole_branch(state_view, block_view, new_epoch, s))
                    .transpose()
            },
        );

        let out = sigs_res?;
        self.reported = out.reported;
        self.reporters = out.reporters;
        self.available = out.available;
        self.counts = out.counts;
        self.reports = out.reports;

        if let Some(s) = safrole_res? {
            self.state.safrole = s.safrole;
            self.block.header.epoch_mark = s.epoch_mark;
            self.block.header.tickets_mark = s.tickets_mark;
        }

        Ok(())
    }

    /// Collect guarantee + assurance ed25519 triples and batch-verify them.
    fn sigs_branch(
        state: &State,
        accounts: &Accounts<S>,
        block: &Block,
        dispute_records: &DisputesRecords,
        new_epoch: bool,
    ) -> Result<SigsOutput> {
        // (p of β') Collect guarantee triples
        let (reported, reporters, mut batch) = if block.extrinsic.guarantees.is_empty() {
            (vec![], vec![], vec![])
        } else {
            guarantee::report(
                state,
                block.header.slot,
                accounts,
                &block.extrinsic.guarantees,
            )?
        };

        // (ρ†) Update availability assignments based on verdicts (10.15)
        let reports = dispute::reports(dispute_records, &state.reports);

        // (W) Collect assurance triples (11.16)
        let (available, counts, a_triples) = assurance::available(
            &reports,
            if new_epoch {
                &state.validators.previous
            } else {
                &state.validators.current
            },
            block.header.parent,
            &block.extrinsic.assurances,
        )?;

        batch.extend(a_triples);
        crypto::ed25519::SigItem::batch_verify(&batch)?;
        Ok(SigsOutput {
            reported,
            reporters,
            available,
            counts,
            reports,
        })
    }

    /// Compute next safrole state via ring-VRF and derive header marks.
    fn safrole_branch(
        state: &State,
        block: &Block,
        new_epoch: bool,
        safrole_in: Safrole,
    ) -> Result<SafroleOutput> {
        let safrole = ticket::safrole(
            state.timeslot,
            block.header.slot,
            state.entropy,
            &state.disputes.offenders,
            safrole_in,
            &state.validators,
            &block.extrinsic.tickets,
        )?;
        let epoch_mark = if new_epoch {
            safrole.epoch_mark(&state.entropy)
        } else {
            None
        };
        let tickets_mark = safrole.tickets_mark(state.timeslot, block.header.slot);
        Ok(SafroleOutput {
            safrole,
            epoch_mark,
            tickets_mark,
        })
    }
}

/// Output of the guarantee/assurance sigs branch.
struct SigsOutput {
    reported: Vec<ReportedWorkPackage>,
    reporters: Vec<Ed25519Public>,
    available: Vec<WorkReport>,
    counts: [u32; CORES_COUNT],
    reports: AvailabilityAssignments,
}

/// Output of the ticket::safrole branch.
struct SafroleOutput {
    safrole: Safrole,
    epoch_mark: Option<EpochMark>,
    tickets_mark: Option<TicketsMark>,
}
