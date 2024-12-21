//! Storage APIs of the state of SpaceJam

use crate::{
    block::history::BlockInfo,
    extrinsic::DisputesRecords,
    misc::{EntropyBuffer, OpaqueHash, Statistics, TimeSlot, ValidatorData},
    state::{key, Safrole, ServiceAccountState, ServiceIndex, State},
    work::report::WorkReport,
    CORES_COUNT, EPOCH_LENGTH,
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
    fn batch_read(&self, keys: Vec<OpaqueHash>) -> Result<Vec<Vec<u8>>>;

    /// Iterate over the storage with a prefix
    fn prefix_iter(
        &self,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = (OpaqueHash, Vec<u8>)>>;

    /// Fetch state from the storage
    ///
    /// We don't decode account data in this batch since it will be too large.
    fn state(&self) -> Result<State> {
        let mut state = State::default();
        let data: Vec<Vec<u8>> = self.batch_read(vec![
            key::AUTHORIZATION_POOLS,
            key::AUTHORIZATION_QUEUE,
            key::RECENT_BLOCKS,
            key::SAFROLE,
            key::DISPUTES,
            key::ENTROPY,
            key::NEXT_VALIDATORS,
            key::CURRENT_VALIDATORS,
            key::PREVIOUS_VALIDATORS,
            key::PENDING_REPORTS,
            key::TIMESLOT,
            key::PRIVILEGED_SERVICE,
            key::STATISTICS,
            key::ACCUMULATION_QUEUE,
            key::ACCUMULATION_HISTORY,
        ])?;

        state.pools = codec::decode(data.get(1).unwrap_or(&vec![]))?;
        state.authorization = codec::decode(data.get(2).unwrap_or(&vec![]))?;
        state.recent_blocks = codec::decode(data.get(3).unwrap_or(&vec![]))?;
        state.safrole = codec::decode(data.get(4).unwrap_or(&vec![]))?;
        state.disputes = codec::decode(data.get(5).unwrap_or(&vec![]))?;
        state.entropy = codec::decode(data.get(6).unwrap_or(&vec![]))?;
        state.validators.next = codec::decode(data.get(7).unwrap_or(&vec![]))?;
        state.validators.current = codec::decode(data.get(8).unwrap_or(&vec![]))?;
        state.validators.previous = codec::decode(data.get(9).unwrap_or(&vec![]))?;
        state.reports = codec::decode(data.get(10).unwrap_or(&vec![]))?;
        state.timeslot = codec::decode(data.get(11).unwrap_or(&vec![]))?;
        state.service = codec::decode(data.get(12).unwrap_or(&vec![]))?;
        state.statistics = codec::decode(data.get(13).unwrap_or(&vec![]))?;
        state.queue = codec::decode(data.get(14).unwrap_or(&vec![]))?;
        state.history = codec::decode(data.get(15).unwrap_or(&vec![]))?;

        // TODO: accumulate account state with `iter()`, requires an update of the trie calculation.
        Ok(state)
    }

    /// Finalize the state
    ///
    /// It's not allowed to save state separately in our system atm for avoiding
    /// uncontrollable dangorous operations, we only provide this method for state
    /// transition, and this should only be called on block finalization.
    ///
    /// TODO: comparing with the current state, only write the updated state.
    fn finalize(&self, state: &State) -> Result<()> {
        self.batch_write(state.accumulate()?)
    }

    /// Fetch the authorization pools from the storage
    fn pools(&self) -> Result<Option<[Vec<OpaqueHash>; CORES_COUNT]>> {
        self.get(key::AUTHORIZATION_POOLS)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode pools: {e}"))
    }

    /// Fetch the authorization queue from the storage
    fn authorization_queue(&self) -> Result<Option<[Vec<OpaqueHash>; CORES_COUNT]>> {
        self.get(key::AUTHORIZATION_QUEUE)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode authorization queue: {e}"))
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
    fn disputes(&self) -> Result<Option<DisputesRecords>> {
        self.get(key::DISPUTES)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode disputes: {e}"))
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
    #[allow(clippy::type_complexity)]
    fn pending_reports(&self) -> Result<Option<[Option<(WorkReport, TimeSlot)>; CORES_COUNT]>> {
        self.get(key::PENDING_REPORTS)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode pending reports: {e}"))
    }

    /// Fetch the timeslot
    fn timeslot(&self) -> Result<Option<TimeSlot>> {
        self.get(key::TIMESLOT)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode timeslot: {e}"))
    }

    /// Fetch the privileged service indices
    fn service(&self) -> Result<Option<ServiceIndex>> {
        self.get(key::PRIVILEGED_SERVICE)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode privileged service: {e}"))
    }

    /// Fetch the activity statistics
    fn statistics(&self) -> Result<Option<Statistics>> {
        self.get(key::STATISTICS)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode statistics: {e}"))
    }

    /// Fetch the accumulation queue
    #[allow(clippy::type_complexity)]
    fn accumulation_queue(
        &self,
    ) -> Result<Option<[(Vec<WorkReport>, Vec<OpaqueHash>); EPOCH_LENGTH as usize]>> {
        self.get(key::ACCUMULATION_QUEUE)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode accumulation queue: {e}"))
    }

    /// Fetch the accumulation history
    fn accumulation_history(&self) -> Result<Option<[Vec<OpaqueHash>; EPOCH_LENGTH as usize]>> {
        self.get(key::ACCUMULATION_HISTORY)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode accumulation history: {e}"))
    }

    /// Fetch the account state
    fn account_state(&self, service: u32) -> Result<Option<ServiceAccountState>> {
        self.get(key::account::state(service))?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode account state: {e}"))
    }

    /// Fetch the account storage
    fn account_storage(&self, service: u32, key: OpaqueHash) -> Result<Option<Vec<u8>>> {
        self.get(key::account::storage(service, key))?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode account storage: {e}"))
    }

    /// Fetch the account preimage
    fn account_preimage(&self, service: u32, key: OpaqueHash) -> Result<Option<Vec<u8>>> {
        self.get(key::account::preimage(service, key))?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode account preimage: {e}"))
    }

    /// Fetch the account lookup
    fn account_lookup(
        &self,
        service: u32,
        lookup: u32,
        key: OpaqueHash,
    ) -> Result<Option<[TimeSlot; 3]>> {
        self.get(key::account::lookup(service, lookup, key))?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode account lookup: {e}"))
    }
}
