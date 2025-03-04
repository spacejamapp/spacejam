//! Runtime utilities of SpaceJam

use crate::{
    extrinsic::{TicketBody, TicketEnvelope},
    Block,
};
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};
use tokio::sync::RwLock;
pub use {
    grandpa::{Grandpa, Head},
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
    pub pool: Pool,

    /// The grandpa of SpaceJam
    pub grandpa: Arc<RwLock<Grandpa>>,

    /// Whether self is a validator.
    pub is_validator: bool,

    // pub metrics:
    /// The attempt number of the current epoch
    attempt: Arc<AtomicU8>,
}

impl<C: Config> Runtime<C> {
    /// Create a new runtime
    pub fn new(validator: C::Validator, storage: C::Storage) -> anyhow::Result<Self> {
        let grandpa = Grandpa::new(&storage)?;
        Ok(Self::new_with_grandpa(validator, storage, grandpa))
    }

    /// Create a new runtime with a grandpa instance
    pub fn new_with_grandpa(
        validator: C::Validator,
        storage: C::Storage,
        grandpa: Grandpa,
    ) -> Self {
        let is_validator = !grandpa
            .grid
            .neighbours(validator.ed25519_public_key())
            .is_empty();
        Self {
            validator,
            storage,
            pool: Default::default(),
            grandpa: Arc::new(RwLock::new(grandpa)),
            attempt: Arc::new(AtomicU8::new(0)),
            is_validator,
        }
    }
    /// Get the next runtime package
    ///
    /// TODO: optimize shared data once we have tests for authoring blocks.
    pub fn next(&self) -> anyhow::Result<(Option<Block>, Option<TicketEnvelope>)> {
        Ok((self.author()?, self.ticket()?))
    }

    /// Author a block
    ///
    /// returns `None` if the current validator is not in the safrole series keys
    pub fn author(&self) -> anyhow::Result<Option<Block>> {
        let safrole = self.storage.safrole()?;
        if !safrole
            .series
            .keys()
            .contains(&self.validator.bandersnatch_public_key())
        {
            return Ok(None);
        }

        let blocks = self.storage.recent_blocks()?;
        let block = blocks
            .last()
            .ok_or(anyhow::anyhow!("genesis block not found"))?;

        let extrinsic = self.pool.collect()?;
        Block::builder()
            .parent(block)?
            .extrinsic(extrinsic)?
            .seal(&self.validator, &self.storage)
            .map(Some)
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

        // check if the sealing series still have seats
        let safrole = self.storage.safrole()?;
        if safrole.series.keys().len() > crate::EPOCH_LENGTH as usize {
            return Ok(None);
        }

        // check if the current validator has exceeded the ticket limit
        let mut attempt = self.attempt.load(Ordering::Relaxed);
        if attempt >= crate::TICKET_ENTRIES_PER_VALIDATOR {
            return Ok(None);
        }

        // use next epoch's validators if the current epoch is over
        let entropy = self.storage.entropy()?;
        let keys = if slot == 0 {
            safrole.validators
        } else {
            self.storage.current_validators()?
        }
        .iter()
        .map(|v| v.bandersnatch)
        .collect::<Vec<_>>();

        // generate a ticket
        attempt += 1;
        self.attempt.store(attempt, Ordering::Relaxed);
        Ok(Some(TicketEnvelope {
            attempt,
            signature: self.validator.bandersnatch_ring_sign(
                &keys,
                &[],
                &TicketBody::message(attempt, &entropy[2]),
            )?,
        }))
    }

    /// Import a block
    pub fn import(&self, block: Vec<u8>) -> anyhow::Result<()> {
        tx::transit(&codec::decode(&block)?, &self.storage, &self.validator)
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
