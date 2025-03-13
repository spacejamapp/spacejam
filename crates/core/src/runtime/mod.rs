//! Runtime utilities of SpaceJam

use crate::{
    block::BlockInfo,
    extrinsic::{TicketBody, TicketEnvelope, TicketsOrKeys},
    safrole::{Safrole, ValidatorData},
    state::key,
    Block, EntropyBuffer, TimeSlot,
};
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};
use storage::{BlockStorage, Branch, KVStorage};
use tokio::sync::RwLock;
pub use {
    grandpa::{Grandpa, Handshake, Head},
    pool::Pool,
    storage::Storage,
    validator::Validator,
};

mod grandpa;
mod pool;
pub mod storage;
pub mod tx;
mod validator;

/// Runtime of SpaceJam
#[derive(Clone)]
pub struct Runtime<C: Config> {
    /// The validator of SpaceJam
    pub validator: C::Validator,

    /// The storage of SpaceJam
    pub storage: C::Storage,

    /// The extrinsic pool of SpaceJam
    pub expool: Pool,

    /// The grandpa of SpaceJam
    pub grandpa: Arc<RwLock<Grandpa>>,

    /// The attempt number of the current epoch
    attempt: Arc<AtomicU8>,
}

impl<C: Config> Runtime<C> {
    /// Create a new runtime with a grandpa instance
    pub fn new(validator: C::Validator, storage: C::Storage) -> Self {
        Self {
            validator,
            storage,
            expool: Default::default(),
            grandpa: Arc::new(RwLock::new(Default::default())),
            attempt: Arc::new(AtomicU8::new(0)),
        }
    }

    /// Get the next runtime package
    ///
    /// TODO: optimize shared data once we have tests for authoring blocks.
    pub async fn next(&self) -> anyhow::Result<(Block, Option<TicketEnvelope>)> {
        let ticket = self.ticket()?;
        if let Some(ticket) = ticket.clone() {
            self.expool
                .tickets
                .lock()
                .await
                .insert((self.validator.bandersnatch_public_key(), ticket));
        }

        let timeslot = crate::block::timeslot()?;
        Ok((self.author(timeslot).await?, ticket))
    }

    /// Get the current pending chain
    pub async fn chain(&self) -> Branch<C::Storage> {
        Branch::checkout(
            &self.storage,
            self.grandpa.read().await.handshake.head.clone(),
        )
    }

    /// Author a block
    pub async fn author(&self, timeslot: TimeSlot) -> anyhow::Result<Block> {
        let chain = self.chain().await;
        let blocks = chain.recent_blocks()?;
        let block = blocks
            .last()
            .ok_or(anyhow::anyhow!("genesis block not found"))?;

        let extrinsic = self.expool.collect().await?;
        Block::builder()
            .parent(block)?
            .extrinsic(extrinsic)?
            .timeslot(timeslot)
            .seal(&self.validator, &chain)
    }

    /// Generate a ticket for the next block (using next timeslot).
    ///
    /// returns `None` if:
    /// - exceed the ticket submission period
    /// - exceed the ticket limit
    pub fn ticket(&self) -> anyhow::Result<Option<TicketEnvelope>> {
        let timeslot = crate::block::timeslot()?;

        // check the ticket submission period
        let slot = timeslot % crate::EPOCH_LENGTH;
        if slot > crate::TICKET_SUBMISSION_PERIOD {
            return Ok(None);
        }

        // reset the attempt number for the new epoch
        if slot == 0 {
            self.attempt.store(0, Ordering::Relaxed);
        }

        // // check if the sealing series still have seats
        // let safrole = self.storage.safrole()?;
        // if safrole.series.keys().len() >= crate::EPOCH_LENGTH as usize {
        //     return Ok(None);
        // }

        // check if the current validator has exceeded the ticket limit
        let attempt = self.attempt.load(Ordering::Relaxed);
        if attempt >= crate::TICKET_ENTRIES_PER_VALIDATOR {
            return Ok(None);
        }

        // TODO: use next epoch's validators if the current epoch is over
        let entropy = self.storage.entropy()?;
        let keys = self
            .storage
            .current_validators()?
            .iter()
            .map(|v| v.bandersnatch)
            .collect::<Vec<_>>();

        // generate a ticket
        //
        // TODO: recheck the selected entropy
        let envelope = TicketEnvelope {
            attempt,
            signature: self.validator.bandersnatch_ring_sign(
                &keys,
                &[],
                &TicketBody::message(attempt, &entropy[2]),
            )?,
        };
        self.attempt.fetch_add(1, Ordering::Relaxed);
        Ok(Some(envelope))
    }

    /// Finalize blocks
    ///
    /// Note that we only store finalized blocks and the blocks authored
    /// by ourselves in our storage.
    #[tracing::instrument(skip_all, level = "debug", name = "Runtime::finalize")]
    pub async fn finalize(&self, block: &mut Block) -> anyhow::Result<()> {
        let prev = self.grandpa.read().await.handshake.head.clone();
        tracing::debug!(
            "previous best block#{}: {}",
            prev.slot,
            hex::encode(prev.hash)
        );

        // 1. transit the global state
        tx::transit(block, &self.storage, &self.validator)?;
        tracing::info!("Finalized block#{}", block.header.slot);

        // 2. save the block to the storage
        self.storage.save_block(block)?;

        // 3. set the latest finalized head
        let head = Head {
            hash: block.hash()?,
            slot: block.header.slot,
        };
        self.storage.set_finalized(&head)?;

        // 4. drop the previous branch
        let branch = Branch::checkout(&self.storage, prev);
        branch.drop()?;

        // 5. update the grandpa state
        let next = if block.header.epoch_mark.is_some() {
            Some(self.storage.next_validators()?)
        } else {
            None
        };
        self.grandpa
            .write()
            .await
            .finalize(block.header.clone(), next)
    }

    /// Import the genesis block
    pub async fn import_genesis(
        &self,
        block: &Block,
        validators: &[ValidatorData],
    ) -> anyhow::Result<()> {
        // 1. save the block to the storage
        self.storage.finalize(block)?;

        // 2. initialize the recent blocks
        let recent: Vec<BlockInfo> = vec![block.header.clone().into()];
        self.storage
            .set(key::RECENT_BLOCKS, codec::encode(&recent)?)?;

        // 3. initialize the validator set
        let encoded = codec::encode(&validators)?;
        self.storage
            .set(key::PREVIOUS_VALIDATORS, encoded.clone())?;
        self.storage.set(key::CURRENT_VALIDATORS, encoded.clone())?;
        self.storage.set(key::NEXT_VALIDATORS, encoded)?;

        // 4. set the safrole state
        let safrole = Safrole {
            series: TicketsOrKeys::Keys(validators.iter().map(|v| v.bandersnatch).collect()),
            validators: validators.to_vec(),
            ..Default::default()
        };
        self.storage.set(key::SAFROLE, codec::encode(&safrole)?)?;

        // 5. set the entropy
        let entropy = EntropyBuffer::default();
        self.storage.set(key::ENTROPY, codec::encode(&entropy)?)?;

        // 5. initialize the grandpa state
        let mut grandpa = self.grandpa.write().await;
        grandpa.grid.next = validators.to_vec();
        grandpa.grid.curr = grandpa.grid.next.clone();
        grandpa.grid.prev = grandpa.grid.curr.clone();
        grandpa.finalize(block.header.clone(), None)?;
        drop(grandpa);

        Ok(())
    }
}

/// The configuration of the runtime
pub trait Config: Send + Sync + 'static {
    /// The storage of the runtime
    type Storage: Storage + Send + Sync + 'static;

    /// The validator of the runtime
    type Validator: Validator + Send + Sync + 'static;
}

impl Config for () {
    type Storage = storage::MemoryDb;
    type Validator = crypto::ed25519::KeyPair;
}
