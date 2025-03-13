//! Memory pool for SpaceJam
#![allow(unused)]

use crate::{
    block,
    extrinsic::{
        AvailAssurance, Culprit, Extrinsic, Fault, Preimage, ReportGuarantee, TicketEnvelope,
        Verdict,
    },
    safrole::ValidatorData,
    BandersnatchPublic, Ed25519Public,
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
    guarantees: Arc<Mutex<Vec<ReportGuarantee>>>,

    /// Assurances
    assurances: Arc<Mutex<Vec<AvailAssurance>>>,

    /// Preimages
    preimages: Arc<Mutex<Vec<Preimage>>>,

    /// Tickets
    pub tickets: Arc<Mutex<HashSet<(BandersnatchPublic, TicketEnvelope)>>>,

    /// Verdicts
    verdicts: Arc<Mutex<Vec<Verdict>>>,

    /// Faults
    faults: Arc<Mutex<Vec<Fault>>>,

    /// Culprits
    culprits: Arc<Mutex<Vec<Culprit>>>,
}

impl Pool {
    /// Sort and insert a ticket into the pool
    pub async fn insert_ticket(&self, ticket: TicketEnvelope, bandersnatch: BandersnatchPublic) {
        let mut tickets = self.tickets.lock().await;
        tickets.insert((bandersnatch, ticket));
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

            // collect tickets
            if timeslot % crate::EPOCH_LENGTH < crate::TICKET_SUBMISSION_PERIOD {
                let mut tickets = self
                    .tickets
                    .lock()
                    .await
                    .clone()
                    .into_iter()
                    .collect::<Vec<_>>();
                self.tickets.lock().await.clear();

                tracing::debug!("collecting {} tickets for epoch: {}", tickets.len(), epoch);
                tickets.sort_by(|a, b| a.0.cmp(&b.0));
                extrinsics.tickets = tickets.into_iter().map(|(_, ticket)| ticket).collect();
            }
        }

        Ok(extrinsics)
    }
}
