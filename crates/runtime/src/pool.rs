//! Memory pool for SpaceJam
#![allow(unused)]

use score::{
    OpaqueHash, block,
    extrinsic::{
        AvailAssurance, Culprit, Extrinsic, Fault, Preimage, ReportGuarantee, Ticket, TicketBody,
        TicketEnvelope, Verdict,
    },
};
use std::{
    collections::{BTreeMap, HashSet},
    sync::Arc,
};
use tokio::sync::Mutex;

/// Memory pool for SpaceJam
///
/// Each of the fields is a designed for concurrent access.
///
/// TODO: redesign the transaction pool, maybe keep the core
/// logic in `runtime` but the optimization in `spacejam`.
#[derive(Clone, Default)]
pub struct Pool {
    /// Guarantees
    guarantees: Vec<ReportGuarantee>,

    /// Assurances
    assurances: Vec<AvailAssurance>,

    /// Preimages
    preimages: Vec<Preimage>,

    /// Tickets
    pub tickets: BTreeMap<u32, HashSet<Ticket>>,

    /// Verdicts
    verdicts: Vec<Verdict>,

    /// Faults
    faults: Vec<Fault>,

    /// Culprits
    culprits: Vec<Culprit>,
}

impl Pool {
    /// Validate the extrinsics in the pool
    ///
    /// 1. remove outdated extrinsics
    /// 2. remove invalid extrinsics
    /// 3. remove duplicated extrinsics
    /// 4. pack the extrinsics into a single extrinsic
    pub async fn collect(&mut self, tickets: Vec<TicketBody>) -> anyhow::Result<Extrinsic> {
        let mut extrinsics = Extrinsic::default();

        // get the current timeslot and epoch
        let timeslot = block::timeslot();
        let epoch = timeslot / score::EPOCH_LENGTH;
        let slot_phase = timeslot % score::EPOCH_LENGTH;

        // Keep only current and next epoch tickets
        let current_epoch = epoch;
        let next_epoch = epoch + 1;
        let initial_count = self.tickets.len();
        self.tickets
            .retain(|&epoch_key, _| epoch_key >= current_epoch && epoch_key <= next_epoch);
        let final_count = self.tickets.len();
        if initial_count != final_count {
            tracing::debug!(
                "cleaned up {} outdated epoch entries, {} remaining",
                initial_count - final_count,
                final_count
            );
        }

        // collect tickets only during submission period (m' < Y)
        if slot_phase >= score::TICKET_SUBMISSION_PERIOD {
            tracing::debug!(
                "skipping ticket collection: slot_phase={} >= submission_period={} (per graypaper: |xttickets| = 0 when m' >= Y)",
                slot_phase,
                score::TICKET_SUBMISSION_PERIOD
            );
            return Ok(Default::default());
        }

        let target_epoch = epoch + 1;
        tracing::debug!(
            "collecting tickets: current_epoch={}, target_epoch={}, slot_phase={}",
            epoch,
            target_epoch,
            slot_phase
        );

        // Get tickets for the target epoch
        let mut collected_tickets = Vec::new();
        if let Some(ticket_set) = self.tickets.get(&target_epoch) {
            collected_tickets.extend(ticket_set.iter().cloned());
        }

        // Remove tickets that are already in the accumulator
        collected_tickets.retain(|ticket| {
            !tickets
                .iter()
                .any(|t| t.id == ticket.id && t.attempt == ticket.envelope.attempt)
        });

        // Sort the tickets by ID
        collected_tickets.sort_by(|a, b| a.id.cmp(&b.id));
        let max_tickets = score::MAX_TICKETS_PER_EXTRINSIC as usize;
        collected_tickets.truncate(max_tickets);

        tracing::trace!(
            "including {} tickets in block for epoch {} (max={})",
            collected_tickets.len(),
            target_epoch,
            max_tickets
        );

        extrinsics.tickets = collected_tickets
            .into_iter()
            .map(|ticket| ticket.envelope)
            .collect();

        Ok(extrinsics)
    }
}
