//! Storage APIs of the state of SpaceJam

use crate::{
    block::history::BlockInfo,
    extrinsic::Judgement,
    misc::{EntropyBuffer, OpaqueHash, Statistics, ValidatorData},
    state::{key, Safrole, State},
    CORES_COUNT,
};
use anyhow::Result;

/// Storage of the state of SpaceJam
///
/// the provided methods in the trait performs storage IO,
/// for higher performance, please reduce the number of IO operations
/// as much as possible.
pub trait Storage {
    /// Set a value in the storage
    fn set(&self, _key: impl AsRef<[u8]>, _value: impl AsRef<[u8]>) -> Result<()>;

    /// Batch write a set of key-value pairs to the storage
    fn batch_write(&self, kvs: Vec<(OpaqueHash, Vec<u8>)>) -> Result<()>;

    /// Get a value from the storage
    fn get(&self, _key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>>;

    /// Batch read a set of key-value pairs from the storage
    fn batch_read(&self, keys: Vec<OpaqueHash>) -> Result<Vec<(OpaqueHash, Vec<u8>)>>;

    /// Fetch state from the storage
    fn state(&self) -> Result<State> {
        todo!("get the state from the storage")
    }

    /// Finalize the state
    ///
    /// It's not allowed to save state seperately in our system atm for avoiding
    /// uncontrolable dangorous operations, we only provide this method for state
    /// transition, and this should only be called on block finalization.
    fn finalize(&self, state: &State) -> Result<()> {
        let kvs = state.accumulate()?;
        for (key, value) in kvs {
            self.set(key, value)?;
        }
        Ok(())
    }

    /// Fetch the authorization pools from the storage
    fn pools(&self) -> Result<Option<[Vec<OpaqueHash>; CORES_COUNT]>> {
        self.get(key::AUTHORIZATION_POOLS)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode pools: {e}"))
    }

    /// Fetch the recent blocks from the storage
    fn recent_blocks(&self) -> Result<Option<Vec<BlockInfo>>> {
        self.get(key::RECENT_BLOCKS)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode recent blocks: {e}"))
    }

    /// Fetch the safrole state
    fn safrole(&self) -> Result<Option<Safrole>> {
        self.get(key::SAFROLE)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode safrole: {e}"))
    }

    /// Fetch the judgements from the storage
    fn judgements(&self) -> Result<Option<Vec<Judgement>>> {
        self.get(key::JUDGEMENTS)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode judgements: {e}"))
    }

    /// Fetch the entropy state
    fn entropy(&self) -> Result<Option<EntropyBuffer>> {
        self.get(key::ENTROPY)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode entropy: {e}"))
    }

    /// Fetch the next validators
    fn next_validators(&self) -> Result<Option<Vec<ValidatorData>>> {
        self.get(key::NEXT_VALIDATORS)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode next validators: {e}"))
    }

    /// Fetch the current validators
    fn current_validators(&self) -> Result<Option<Vec<ValidatorData>>> {
        self.get(key::CURRENT_VALIDATORS)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode current validators: {e}"))
    }

    /// Fetch the previous validators
    fn previous_validators(&self) -> Result<Option<Vec<ValidatorData>>> {
        self.get(key::PREVIOUS_VALIDATORS)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode previous validators: {e}"))
    }

    /// Fetch the pending reports
    fn pending_reports(&self) -> Result<Option<Vec<()>>> {
        self.get(key::PENDING_REPORTS)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode pending reports: {e}"))
    }

    /// Fetch the timeslot
    fn timeslot(&self) -> Result<Option<u64>> {
        self.get(key::TIMESLOT)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode timeslot: {e}"))
    }

    /// Fetch the privileged service indices
    fn privileged_service(&self) -> Result<Option<Vec<u64>>> {
        self.get(key::PRIVILEGED_SERVICE)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode privileged service: {e}"))
    }

    /// Fetch the activity statistics
    fn statistics(&self) -> Result<Option<Vec<Statistics>>> {
        self.get(key::STATISTICS)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode statistics: {e}"))
    }

    /// Fetch the accumulation queue
    fn accumulation_queue(&self) -> Result<Option<Vec<()>>> {
        self.get(key::ACCUMULATION_QUEUE)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode accumulation queue: {e}"))
    }

    /// Fetch the accumulation history
    fn accumulation_history(&self) -> Result<Option<Vec<()>>> {
        self.get(key::ACCUMULATION_HISTORY)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode accumulation history: {e}"))
    }

    /// Fetch the account state
    fn account_state(&self, service: u32) -> Result<Option<()>> {
        self.get(key::account::state(service))?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode account state: {e}"))
    }

    /// Fetch the account storage
    fn account_storage(&self, service: u32, key: OpaqueHash) -> Result<Option<()>> {
        self.get(key::account::storage(service, key))?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode account storage: {e}"))
    }

    /// Fetch the account preimage
    fn account_preimage(&self, service: u32, key: OpaqueHash) -> Result<Option<()>> {
        self.get(key::account::preimage(service, key))?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode account preimage: {e}"))
    }

    /// Fetch the account lookup
    fn account_lookup(&self, service: u32, lookup: u32, key: OpaqueHash) -> Result<Option<()>> {
        self.get(key::account::lookup(service, lookup, key))?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode account lookup: {e}"))
    }
}
