//! Authoring service

use crate::{
    block::{self, Block, Header},
    extrinsic::{TicketBody, TicketEnvelope, TicketsOrKeys},
    runtime::{storage::SyncStorage, tx, Runtime, Storage, Validator},
    BandersnatchPublic, EntropyBuffer, OpaqueHash, TimeSlot,
};
use std::{
    collections::VecDeque,
    sync::atomic::{AtomicU8, Ordering},
};

/// Authoring context
pub struct Author<'a, C: crate::runtime::Config> {
    /// The runtime
    pub runtime: &'a Runtime<C>,

    /// The local validator
    pub me: BandersnatchPublic,

    /// The current timeslot
    pub timeslot: TimeSlot,

    /// The ticket id
    tickets: Vec<OpaqueHash>,

    /// The slots that we are authoring
    slots: VecDeque<TimeSlot>,

    /// The attempt number of the current epoch
    attempt: AtomicU8,
}

impl<'a, C: crate::runtime::Config> Author<'a, C> {
    /// Create a new authoring context
    pub fn new(runtime: &'a Runtime<C>) -> Self {
        let me = runtime.validator.bandersnatch_public_key();

        Self {
            runtime,
            me,
            timeslot: Default::default(),
            slots: Default::default(),
            attempt: Default::default(),
            tickets: Default::default(),
        }
    }

    /// do authoring
    pub async fn next(&mut self) -> anyhow::Result<(Option<Header>, Option<TicketEnvelope>)> {
        let timeslot = block::timeslot()?;
        self.on_timeslot(timeslot).await
    }

    /// Run to the next timeslot
    pub async fn on_timeslot(
        &mut self,
        timeslot: u32,
    ) -> anyhow::Result<(Option<Header>, Option<TicketEnvelope>)> {
        let slot = timeslot % crate::EPOCH_LENGTH;
        let mut next = (None, None);

        // TODO: handle this rotation on the storage interface
        // for syncing interfaces.
        if slot == 0 {
            self.on_new_epoch().await?;
        }

        // 1. check generating tickets
        //
        // Note that we only generate tickets after two slots.
        if slot > 2 {
            if let Some((id, envelope)) = self.ticket().await? {
                next.1 = Some(envelope);
                self.tickets.push(id);
            }
        }

        // 2. update the timeslot
        self.timeslot = timeslot;

        // 3. check authoring blocks
        if self.slots.contains(&slot) {
            next.0 = Some(self.author().await?);
            self.slots.pop_front();
        }

        Ok(next)
    }

    /// on new epoch
    pub async fn on_new_epoch(&mut self) -> anyhow::Result<()> {
        self.runtime.storage.on_new_epoch()?;

        // 1. reset the attempt number
        self.attempt.store(0, Ordering::Relaxed);

        // 2. clean the ticket cache
        self.runtime.expool.tickets.lock().await.clear();

        // 3. update the authoring slots
        let mut slots = VecDeque::new();
        let mut fallback = false;
        match self.series()? {
            TicketsOrKeys::Tickets(tickets) => {
                for (i, ticket) in tickets.iter().enumerate() {
                    if self.tickets.contains(&ticket.id) {
                        tracing::debug!("assigned slot#{i} with ticket#{}", hex::encode(ticket.id));
                        slots.push_back(i as TimeSlot);
                    }
                }
            }
            TicketsOrKeys::Keys(keys) => {
                fallback = true;
                for (i, author) in keys.iter().enumerate() {
                    if author == &self.me {
                        slots.push_back(i as TimeSlot);
                    }
                }
            }
        }
        self.slots = slots;
        tracing::info!(
            "using {} keys, authoring slots: {:?}",
            if fallback { "fallback" } else { "safrole" },
            self.slots
        );

        self.tickets.clear();
        Ok(())
    }

    /// Author a block
    pub async fn author(&self) -> anyhow::Result<Header> {
        let keys = self.keys()?;
        // 1. get the last block
        let blocks = self.runtime.storage.recent_blocks()?;
        let parent = blocks
            .last()
            .ok_or(anyhow::anyhow!("genesis block not found"))?;

        // 2. collect the extrinsics
        let envelopes = self.runtime.storage.safrole()?.accumulator;
        let extrinsic = self.runtime.expool.collect(envelopes).await?;

        // 3. init the builder
        let mut builder = Block::builder()
            .parent(parent)?
            .extrinsic(extrinsic)?
            .timeslot(self.timeslot);

        // 4. set the author index
        let author_index = keys
            .iter()
            .position(|v| *v == self.me)
            .ok_or_else(|| anyhow::anyhow!("validator not present in the current validator set"))?;
        builder = builder.author_index(author_index as u16);

        // 5. simulate the block
        tx::simulate(&mut builder, &self.runtime.storage, &self.runtime.validator)?;

        // 6. seal the block
        let block = builder.seal(
            &self.runtime.validator,
            &keys,
            self.series()?,
            self.entropy()?,
        )?;

        // 7. save the block to the fork storage
        self.save_block(block).await
    }

    /// Generate a ticket
    pub async fn ticket(&self) -> anyhow::Result<Option<(OpaqueHash, TicketEnvelope)>> {
        let epoch = self.timeslot / crate::EPOCH_LENGTH;

        // 1. check if the current validator has exceeded the ticket limit
        let attempt = self.attempt.load(Ordering::Relaxed);
        if attempt >= crate::TICKET_ENTRIES_PER_VALIDATOR {
            return Ok(None);
        }

        // 2. generate a ticket
        let entropy = self.entropy()?;
        let next_keys = self.next_keys()?;
        let envelope = TicketEnvelope {
            attempt,
            signature: self.runtime.validator.bandersnatch_ring_sign(
                &next_keys,
                &[],
                &TicketBody::message(attempt, &entropy[2]),
            )?,
        };

        tracing::info!(
            "generated ticket#{} with entropy: 0x{}",
            attempt,
            hex::encode(entropy[2].as_ref())
        );

        // 3. insert the ticket into the pool
        self.attempt.fetch_add(1, Ordering::Relaxed);
        let id = self.insert_ticket(epoch, envelope.clone()).await?;
        Ok(Some((id, envelope)))
    }

    /// Sort and insert a ticket into the pool
    pub async fn insert_ticket(
        &self,
        _epoch: u32,
        ticket: TicketEnvelope,
    ) -> anyhow::Result<OpaqueHash> {
        let keys = self.next_keys()?;
        let entropy = self.entropy()?;
        let verifier = crypto::ring::verifier(keys);
        let id = match verifier.ring_vrf_verify(
            &TicketBody::message(ticket.attempt, &entropy[2]),
            &[],
            &ticket.signature,
        ) {
            Ok(id) => id,
            Err(e) => {
                anyhow::bail!(
                    "invalid ticket#{} with entropy: 0x{}, {e}",
                    ticket.attempt,
                    hex::encode(entropy[2].as_ref()),
                );
            }
        };

        let mut tickets = self.runtime.expool.tickets.lock().await;
        tickets.insert((id, ticket));
        Ok(id)
    }

    /// Save a block to the fork storage
    async fn save_block(&self, block: Block) -> anyhow::Result<Header> {
        self.runtime.storage.set_block(&block)?;
        self.runtime
            .grandpa
            .write()
            .await
            .add_leaf(block.header.clone())?;
        Ok(block.header)
    }

    /// Get the bandersnatch keys of the current validators
    fn keys(&self) -> anyhow::Result<Vec<BandersnatchPublic>> {
        let validators = self.runtime.storage.current_validators()?;
        Ok(validators
            .iter()
            .map(|v| v.bandersnatch)
            .collect::<Vec<_>>())
    }

    /// Get the bandersnatch keys for the next validators
    fn next_keys(&self) -> anyhow::Result<Vec<BandersnatchPublic>> {
        let validators = self.runtime.storage.next_validators()?;
        Ok(validators
            .iter()
            .map(|v| v.bandersnatch)
            .collect::<Vec<_>>())
    }

    /// Get the entropy
    fn entropy(&self) -> anyhow::Result<EntropyBuffer> {
        self.runtime.storage.entropy()
    }

    /// Get the series
    fn series(&self) -> anyhow::Result<TicketsOrKeys> {
        self.runtime.storage.series()
    }
}
