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

            // collect tickets
            if timeslot % score::EPOCH_LENGTH < score::TICKET_SUBMISSION_PERIOD {
                let mut envelopes = self
                    .tickets
                    .lock()
                    .await
                    .clone()
                    .into_iter()
                    .collect::<Vec<_>>();

                // remove the tickets that are already in the pool
                envelopes.retain(|ticket| {
                    !tickets
                        .iter()
                        .any(|t| t.id == ticket.id && t.attempt == ticket.envelope.attempt)
                });

                // sort the envelopes by the id
                envelopes.sort_by(|a, b| a.id.cmp(&b.id));
                envelopes.truncate(3);
                extrinsics.tickets = envelopes
                    .into_iter()
                    .map(|ticket| ticket.envelope)
                    .collect();
            }
        }

        Ok(extrinsics)
    }
}
