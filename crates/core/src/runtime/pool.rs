//! Memory pool for SpaceJam
#![allow(unused)]

use crate::{
    block,
    extrinsic::{
        AvailAssurance, Culprit, Extrinsic, Fault, Preimage, ReportGuarantee, TicketEnvelope,
        Verdict,
    },
};
use std::{collections::BTreeMap, sync::Arc};
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
    tickets: Arc<Mutex<BTreeMap<u32, Vec<TicketEnvelope>>>>,

    /// Verdicts
    verdicts: Arc<Mutex<Vec<Verdict>>>,

    /// Faults
    faults: Arc<Mutex<Vec<Fault>>>,

    /// Culprits
    culprits: Arc<Mutex<Vec<Culprit>>>,
}

impl Pool {
    /// Insert a ticket into the pool
    pub async fn insert_ticket(&self, epoch: u32, ticket: TicketEnvelope) -> anyhow::Result<()> {
        self.tickets
            .lock()
            .await
            .entry(epoch)
            .or_insert_with(Vec::new)
            .push(ticket);
        Ok(())
    }

    /// Validate the extrinsics in the pool
    ///
    /// 1. remove outdated extrinsics
    /// 2. remove invalid extrinsics
    /// 3. remove duplicated extrinsics
    /// 4. pack the extrinsics into a single extrinsic
    pub async fn collect(&self) -> anyhow::Result<Extrinsic> {
        let mut extrinsics = Extrinsic::default();

        {
            // collect tickets
            let timeslot = block::timeslot()?;
            let epoch = timeslot / crate::EPOCH_LENGTH;

            if timeslot % crate::EPOCH_LENGTH < crate::TICKET_SUBMISSION_PERIOD {
                let mut entry = self.tickets.lock().await.entry(epoch).or_default().clone();
                let tickets = entry.clone();
                self.tickets.lock().await.clear();
                extrinsics.tickets = tickets;
            }
        }

        Ok(extrinsics)
    }
}
