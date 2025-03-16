//! Authoring service

use crate::{
    block::{self, Block, Header},
    extrinsic::{TicketBody, TicketEnvelope, TicketsOrKeys},
    runtime::{storage::BlockStorage, tx, Head, Runtime, Storage, Validator},
    safrole::ValidatorsData,
    BandersnatchPublic, EntropyBuffer, TimeSlot,
};
use std::{
    collections::VecDeque,
    sync::atomic::{AtomicU8, Ordering},
    time::Duration,
};

/// Authoring context
pub struct Author<'a, C: crate::runtime::Config> {
    /// The runtime
    runtime: &'a Runtime<C>,

    /// The entropy buffer
    entropy: EntropyBuffer,

    /// The local validator
    pub me: BandersnatchPublic,

    /// The current timeslot
    pub timeslot: TimeSlot,

    /// The current validators
    pub validators: ValidatorsData,

    /// the current series
    pub series: TicketsOrKeys,

    /// The slots that we are authoring
    slots: VecDeque<TimeSlot>,

    /// The keys of the current series
    keys: Vec<BandersnatchPublic>,

    /// The attempt number of the current epoch
    attempt: AtomicU8,
}

impl<'a, C: crate::runtime::Config> Author<'a, C> {
    /// Create a new authoring context
    pub fn new(runtime: &'a Runtime<C>) -> Self {
        let me = runtime.validator.bandersnatch_public_key();

        Self {
            runtime,
            entropy: Default::default(),
            me,
            timeslot: Default::default(),
            validators: Default::default(),
            series: Default::default(),
            slots: Default::default(),
            keys: Default::default(),
            attempt: Default::default(),
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
        if slot == 0 {
            self.on_new_epoch().await?;
        }
        let mut next = (None, None);

        // 1. wait for the next epoch if we are not a validator
        if !self.validators.iter().any(|v| v.bandersnatch == self.me) {
            let duration =
                (crate::EPOCH_LENGTH - (timeslot % crate::EPOCH_LENGTH)) * crate::SLOT_PERIOD;
            tracing::info!("not a validator, sleeping {duration}s for the next epoch");
            tokio::time::sleep(Duration::from_secs(duration as u64)).await;
            return Ok(next);
        }

        // 2. check generating tickets
        next.1 = self.ticket().await?;

        // 3. update the timeslot
        self.timeslot = timeslot;

        // 4. check authoring blocks
        if self.slots.contains(&slot) {
            next.0 = Some(self.author().await?);
            self.slots.pop_front();
        }

        Ok(next)
    }

    /// on new epoch
    pub async fn on_new_epoch(&mut self) -> anyhow::Result<()> {
        // 1. update the validators
        self.validators = self.runtime.storage.current_validators()?;

        // 2. update the keys
        self.keys = self
            .validators
            .iter()
            .map(|v| v.bandersnatch)
            .collect::<Vec<_>>();

        // 3. check if we are in the fallback mode
        let safrole = self.runtime.chain().await.safrole()?;
        self.series = safrole.series;

        // 4. update the authoring slots
        let mut slots = VecDeque::new();
        match &self.series {
            TicketsOrKeys::Tickets(_tickets) => {
                tracing::warn!("tickets series is not supported yet, stop authoring");
                slots = VecDeque::new();
            }
            TicketsOrKeys::Keys(keys) => {
                for (i, author) in keys.iter().enumerate() {
                    if author == &self.me {
                        slots.push_back(i as TimeSlot);
                    }
                }
            }
        }
        self.slots = slots;

        // 5. update the entropy buffer
        self.entropy = self.runtime.storage.entropy()?;

        // 6. reset the attempt number
        self.attempt.store(0, Ordering::Relaxed);

        // 7. clean the ticket cache
        self.runtime.expool.tickets.lock().await.clear();

        Ok(())
    }

    /// Author a block
    pub async fn author(&self) -> anyhow::Result<Header> {
        let keys = self.keys()?;
        // 1. get the last block
        let chain = self.runtime.chain().await;
        let blocks = chain.recent_blocks()?;
        let parent = blocks
            .last()
            .ok_or(anyhow::anyhow!("genesis block not found"))?;

        // 2. collect the extrinsics
        let extrinsic = self.runtime.expool.collect().await?;

        // 2. init the builder
        let mut builder = Block::builder()
            .parent(parent)?
            .extrinsic(extrinsic)?
            .timeslot(self.timeslot);

        // 3. set the author index
        let author_index = keys
            .iter()
            .position(|v| *v == self.me)
            .ok_or_else(|| anyhow::anyhow!("validator not present in the current validator set"))?;
        builder = builder.author_index(author_index as u16);

        // 4. simulate the block
        tx::simulate(
            &mut builder,
            &self.runtime.chain().await,
            &self.runtime.validator,
        )?;

        // 5. seal the block
        let block = builder.seal(
            &self.runtime.validator,
            &keys,
            self.series()?,
            self.entropy()?,
        )?;

        // 6. save the block to the fork storage
        self.save_block(block).await
    }

    /// Generate a ticket
    pub async fn ticket(&self) -> anyhow::Result<Option<TicketEnvelope>> {
        // 1. check if the current validator has exceeded the ticket limit
        let attempt = self.attempt.load(Ordering::Relaxed);
        if attempt >= crate::TICKET_ENTRIES_PER_VALIDATOR {
            return Ok(None);
        }

        // 2. generate a ticket
        let envelope = TicketEnvelope {
            attempt,
            signature: self.runtime.validator.bandersnatch_ring_sign(
                &self.keys,
                &[],
                &TicketBody::message(attempt, &self.entropy[2]),
            )?,
        };
        self.attempt.fetch_add(1, Ordering::Relaxed);
        tracing::trace!(
            "generated ticket#{} with entropy: 0x{}, validators set: {:#?}",
            attempt,
            hex::encode(self.entropy[2].as_ref()),
            self.keys
                .iter()
                .map(|v| hex::encode(v.as_ref()))
                .collect::<Vec<_>>()
        );

        // 3. insert the ticket into the pool
        self.insert_ticket(envelope.clone()).await?;
        Ok(Some(envelope))
    }

    /// Sort and insert a ticket into the pool
    pub async fn insert_ticket(&self, ticket: TicketEnvelope) -> anyhow::Result<()> {
        let keys = self.keys()?;
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
        Ok(())
    }

    /// Save a block to the fork storage
    async fn save_block(&self, block: Block) -> anyhow::Result<Header> {
        let chain = self.runtime.chain().await;

        // save the block to the storage
        let head = Head {
            hash: block.hash()?,
            slot: block.header.slot,
        };
        chain.save_block(&block)?;
        chain.set_finalized(&head)?;

        // save the header to the grandpa
        {
            let mut grandpa = self.runtime.grandpa.write().await;
            grandpa.add_leaf(block.header.clone())?;
        }

        // transit the state
        tx::transit(block.clone(), &chain, &self.runtime.validator)?;
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

    /// Get the entropy
    fn entropy(&self) -> anyhow::Result<EntropyBuffer> {
        self.runtime.storage.entropy()
    }

    /// Get the series
    fn series(&self) -> anyhow::Result<TicketsOrKeys> {
        let safrole = self.runtime.storage.safrole()?;
        Ok(safrole.series)
    }
}
