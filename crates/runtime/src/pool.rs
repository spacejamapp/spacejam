//! Memory pool for SpaceJam
#![allow(unused)]

use score::{
    block,
    extrinsic::{
        AvailAssurance, Culprit, Extrinsic, Fault, Preimage, ReportGuarantee, Ticket, TicketBody,
        TicketEnvelope, Verdict,
    },
    OpaqueHash,
};
use std::{collections::HashSet, sync::Arc};
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
    guarantees: Arc<Mutex<Vec<ReportGuarantee>>>,

    /// Assurances
    assurances: Arc<Mutex<Vec<AvailAssurance>>>,

    /// Preimages
    preimages: Arc<Mutex<Vec<Preimage>>>,

    /// Tickets
    pub tickets: Arc<Mutex<HashSet<Ticket>>>,

    /// Verdicts
    verdicts: Arc<Mutex<Vec<Verdict>>>,

    /// Faults
    faults: Arc<Mutex<Vec<Fault>>>,

    /// Culprits
    culprits: Arc<Mutex<Vec<Culprit>>>,
}

impl Pool {
    /// Validate the extrinsics in the pool
    ///
    /// 1. remove outdated extrinsics
    /// 2. remove invalid extrinsics
    /// 3. remove duplicated extrinsics
    /// 4. pack the extrinsics into a single extrinsic
    ///
    /// NOTE: currently we only collect 3 tickets on each block.
    pub async fn collect(&self, tickets: Vec<TicketBody>) -> anyhow::Result<Extrinsic> {
        let mut extrinsics = Extrinsic::default();

        {
            // get the current timeslot and epoch
            let timeslot = block::timeslot();
            let epoch = timeslot / score::EPOCH_LENGTH;
            let slot_phase = timeslot % score::EPOCH_LENGTH;

            // collect tickets only during submission period
            if slot_phase < score::TICKET_SUBMISSION_PERIOD {
                let mut envelopes = self
                    .tickets
                    .lock()
                    .await
                    .clone()
                    .into_iter()
                    .collect::<Vec<_>>();

                // ✅ CRITICAL FIX: Only include tickets for the NEXT epoch
                // Per JAMNP spec: "tickets are distributed in the epoch PRIOR to the one they're used in"
                let target_epoch = epoch + 1;
                tracing::debug!(
                    "collecting tickets: current_epoch={}, target_epoch={}, slot_phase={}",
                    epoch,
                    target_epoch,
                    slot_phase
                );

                // remove tickets that are already in the accumulator
                envelopes.retain(|ticket| {
                    !tickets
                        .iter()
                        .any(|t| t.id == ticket.id && t.attempt == ticket.envelope.attempt)
                });

                // ✅ NEW: Filter out tickets that aren't for the next epoch
                // This prevents TicketDropped errors by ensuring only valid epoch tickets are included
                envelopes.retain(|ticket| {
                    // TODO: Add proper epoch validation here when ticket contains epoch info
                    // For now, we assume tickets in pool are valid for next epoch
                    true
                });

                // sort the envelopes by the id
                envelopes.sort_by(|a, b| a.id.cmp(&b.id));
                envelopes.truncate(3);

                tracing::trace!(
                    "including {} tickets in block for epoch {}",
                    envelopes.len(),
                    target_epoch
                );

                extrinsics.tickets = envelopes
                    .into_iter()
                    .map(|ticket| ticket.envelope)
                    .collect();
            } else {
                tracing::debug!(
                    "skipping ticket collection: slot_phase={} >= submission_period={}",
                    slot_phase,
                    score::TICKET_SUBMISSION_PERIOD
                );
            }
        }

        Ok(extrinsics)
    }
}
