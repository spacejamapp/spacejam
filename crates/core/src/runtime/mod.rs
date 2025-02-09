//! Runtime utilities of SpaceJam

use crate::{
    extrinsic::{TicketBody, TicketEnvelope},
    Block,
};
use std::sync::atomic::{AtomicU8, Ordering};
pub use {storage::Storage, validator::Validator};

pub mod storage;
pub mod tx;
mod validator;

/// Runtime of SpaceJam
///
/// TODO: maybe holds the latest state in memory?
pub struct Runtime<S: Storage, V: Validator> {
    /// The validator of SpaceJam
    pub validator: V,

    /// The storage of SpaceJam
    pub storage: S,

    /// The attempt number of the current epoch
    attempt: AtomicU8,
}

impl<S: Storage, V: Validator> Runtime<S, V> {
    /// Create a new runtime
    pub fn new(validator: V, storage: S) -> Self {
        Self {
            validator,
            storage,
            attempt: AtomicU8::new(0),
        }
    }

    /// Author a block
    ///
    /// returns `None` if the current validator is not in the safrole series keys
    pub async fn try_author(&self) -> anyhow::Result<Option<Block>> {
        let safrole = self.storage.safrole()?;
        if !safrole
            .series
            .keys()
            .contains(&self.validator.bandersnatch_public_key())
        {
            return Ok(None);
        }

        let block = self.storage.recent_blocks()?;
        let Some(block) = block.and_then(|b| b.last().cloned()) else {
            anyhow::bail!("genesis block not found");
        };

        Block::builder()
            .parent(&block)?
            .seal(&self.validator, &self.storage)
            .map(Some)
    }

    /// Generate a ticket for the next block (using next timeslot).
    ///
    /// returns `None` if:
    /// - exceed the ticket submission period
    /// - exceed the ticket limit
    pub async fn ticket(&self) -> anyhow::Result<Option<TicketEnvelope>> {
        let timeslot = crate::block::timeslot()? + 1;

        // check the ticket submission period
        let slot = timeslot % crate::EPOCH_LENGTH;
        if slot > crate::TICKET_SUBMISSION_PERIOD {
            return Ok(None);
        }

        // check if the sealing series still have seats
        //
        // TODO: this could be cached in memory or sort of cache db.
        let safrole = self.storage.safrole()?;
        if safrole.series.keys().len() > crate::EPOCH_LENGTH as usize {
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
        self.attempt.fetch_add(1, Ordering::Relaxed);
        let attempt = self.attempt.load(Ordering::Relaxed);
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
