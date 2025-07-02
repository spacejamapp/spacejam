//! Storage APIs of the state of SpaceJam

use crate::storage::{sync, Column, KVStorage};
use anyhow::{Context, Result};
use crypto::merkle;
use score::{
    block::BlockInfo,
    extrinsic::DisputesRecords,
    safrole::{Safrole, ValidatorsData},
    service::{AvailabilityAssignments, Privileges, ServiceAccount, ServiceData, WorkReport},
    state::{account, key, ServiceField, State, StateKey, StateKeyInfo, StateKeyLike},
    statistic::Statistics,
    EntropyBuffer, OpaqueHash, ServiceId, TimeSlot, CORES_COUNT, EPOCH_LENGTH,
};

/// Storage of the state of SpaceJam
///
/// the provided methods in the trait performs storage IO,
/// for higher performance, please reduce the number of IO operations
/// as much as possible.
pub trait StateStorage: KVStorage {
    /// Check if the storage is empty
    fn is_empty(&self) -> bool {
        let timeslot = self.state_get(key::TIMESLOT);
        if let Ok(Some(timeslot)) = timeslot {
            codec::decode::<TimeSlot>(timeslot.as_ref()).is_err()
        } else {
            true
        }
    }

    /// Get the state from the storage
    fn state_get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        self.get(Column::State, key)
    }

    /// Get the state from the storage
    fn state_set(&self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<()> {
        self.set(Column::State, key, value)
    }

    /// Get the state from the storage
    fn state_iter(&self) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        self.iter(Column::State)
    }

    fn state_prefix_iter(
        &self,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        self.prefix_iter(Column::State, prefix)
    }

    fn state_batch_read(&self, keys: Vec<Vec<u8>>) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        self.batch_read(Column::State, keys)
    }

    /// Fetch state from the storage
    ///
    /// We don't decode account data in this batch since it will be too large.
    fn state(&self) -> Result<State> {
        let mut state = State::default();
        let data: Vec<Vec<u8>> = self
            .state_batch_read(
                vec![
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
                ]
                .into_iter()
                .map(|k| k.to_vec())
                .collect::<Vec<_>>(),
            )?
            .into_iter()
            .map(|(_, v)| v)
            .collect::<Vec<_>>();

        state.pools = codec::decode(&data[0]).unwrap_or_default();
        state.authorization = codec::decode(&data[1]).unwrap_or_default();
        state.recent_blocks = codec::decode(&data[2]).unwrap_or_default();
        state.safrole = codec::decode(&data[3]).unwrap_or_default();
        state.disputes = codec::decode(&data[4]).unwrap_or_default();
        state.entropy = codec::decode(&data[5]).unwrap_or_default();
        state.validators.next = codec::decode(&data[6]).unwrap_or_default();
        state.validators.current = codec::decode(&data[7]).unwrap_or_default();
        state.validators.previous = codec::decode(&data[8]).unwrap_or_default();
        state.reports = codec::decode(&data[9]).unwrap_or_default();
        state.timeslot = codec::decode(&data[10]).unwrap_or_default();
        state.privileges = codec::decode(&data[11]).unwrap_or_default();
        state.statistics = codec::decode(&data[12]).unwrap_or_default();
        state.queue = codec::decode(&data[13]).unwrap_or_default();
        state.history = codec::decode(&data[14]).unwrap_or_default();
        Ok(state)
    }

    /// Calculate the root of the state from storage.
    ///
    /// FIXME: it is not ideal to store all data in memory
    /// for calculating the root.
    fn root(&self) -> Result<OpaqueHash> {
        let mut kvs = Vec::new();
        for pair in self.state_iter()? {
            let (k, v) = pair?;
            if k.starts_with(sync::SYNC) || k.len() != 31 {
                continue;
            }

            let mut key = [0; 31];
            let len = k.len().min(31);
            key[..len].copy_from_slice(&k[..len]);
            kvs.push((key, v));
        }

        Ok(merkle::trie31(&kvs))
    }

    /// Fetch the authorization pools from the storage
    fn pools(&self) -> Result<Option<[Vec<OpaqueHash>; CORES_COUNT]>> {
        self.state_get(key::AUTHORIZATION_POOLS)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode pools: {e}"))
    }

    /// Fetch the authorization queue from the storage
    fn authorization_queue(&self) -> Result<Option<[Vec<OpaqueHash>; CORES_COUNT]>> {
        self.state_get(key::AUTHORIZATION_QUEUE)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode authorization queue: {e}"))
    }

    /// Fetch the recent blocks from the storage
    fn recent_blocks(&self) -> Result<Vec<BlockInfo>> {
        codec::decode(
            &self
                .state_get(key::RECENT_BLOCKS)?
                .ok_or(anyhow::anyhow!("recent blocks not found"))?,
        )
        .context("failed to decode recent blocks")
    }

    /// Fetch the safrole state
    fn safrole(&self) -> Result<Safrole> {
        codec::decode(
            &self
                .state_get(key::SAFROLE)?
                .ok_or(anyhow::anyhow!("safrole not found"))?,
        )
        .context("failed to decode safrole")
    }

    /// Fetch the judgements from the storage
    fn disputes(&self) -> Result<Option<DisputesRecords>> {
        self.state_get(key::DISPUTES)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode disputes: {e}"))
    }

    /// Fetch the entropy state
    fn entropy(&self) -> Result<EntropyBuffer> {
        codec::decode(
            &self
                .state_get(key::ENTROPY)?
                .ok_or(anyhow::anyhow!("entropy not found"))?,
        )
        .context("failed to decode entropy")
    }

    /// Fetch the next validators
    fn next_validators(&self) -> Result<ValidatorsData> {
        codec::decode(
            &self
                .state_get(key::NEXT_VALIDATORS)?
                .ok_or(anyhow::anyhow!("next validators not found"))?,
        )
        .context("failed to decode next validators")
    }

    /// Fetch the current validators
    fn current_validators(&self) -> Result<ValidatorsData> {
        codec::decode(
            &self
                .state_get(key::CURRENT_VALIDATORS)?
                .ok_or(anyhow::anyhow!("current validators not found"))?,
        )
        .context("failed to decode current validators")
    }

    /// Fetch the previous validators
    fn previous_validators(&self) -> Result<ValidatorsData> {
        codec::decode(
            &self
                .state_get(key::PREVIOUS_VALIDATORS)?
                .ok_or(anyhow::anyhow!("previous validators not found"))?,
        )
        .context("failed to decode previous validators")
    }

    /// Fetch the pending reports
    fn pending_reports(&self) -> Result<AvailabilityAssignments> {
        self.state_get(key::PENDING_REPORTS)?
            .map(|value| codec::decode(&value))
            .ok_or(anyhow::anyhow!("pending reports not found"))?
            .map_err(|e| anyhow::anyhow!("failed to decode pending reports: {e}"))
    }

    /// Fetch the timeslot
    fn timeslot(&self) -> Result<TimeSlot> {
        codec::decode(
            &self
                .state_get(key::TIMESLOT)?
                .ok_or(anyhow::anyhow!("timeslot not found"))?,
        )
        .context("failed to decode timeslot")
    }

    /// Fetch the privileged service indices
    fn privileges(&self) -> Result<Privileges> {
        self.state_get(key::PRIVILEGED_SERVICE)?
            .map(|value| codec::decode(&value))
            .ok_or(anyhow::anyhow!("privileged service not found"))?
            .map_err(|e| anyhow::anyhow!("failed to decode privileged service: {e}"))
    }

    /// Fetch the activity statistics
    fn statistics(&self) -> Result<Statistics> {
        self.state_get(key::STATISTICS)?
            .map(|value| codec::decode(&value))
            .ok_or(anyhow::anyhow!("statistics not found"))?
            .map_err(|e| anyhow::anyhow!("failed to decode statistics: {e}"))
    }

    /// Fetch the accumulation queue
    #[allow(clippy::type_complexity)]
    fn accumulation_queue(
        &self,
    ) -> Result<Option<[(Vec<WorkReport>, Vec<OpaqueHash>); EPOCH_LENGTH as usize]>> {
        self.state_get(key::ACCUMULATION_QUEUE)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode accumulation queue: {e}"))
    }

    /// Fetch the accumulation history
    fn accumulation_history(&self) -> Result<Option<[Vec<OpaqueHash>; EPOCH_LENGTH as usize]>> {
        self.state_get(key::ACCUMULATION_HISTORY)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode accumulation history: {e}"))
    }

    /// Fetch the account
    ///
    /// FIXME: this is not efficient, we should fetch a set of accounts in one batch.
    fn account(&self, index: u32) -> Result<ServiceAccount> {
        let mut account = ServiceAccount::default();
        for item in self.state_iter()? {
            let (key, value) = item?;
            match key.as_state_key().info() {
                StateKey::Account {
                    service,
                    field: ServiceField::Data,
                } => {
                    if service != index {
                        continue;
                    }

                    account.code = value[..32].try_into()?;
                    account.balance = u64::from_le_bytes(value[32..40].try_into()?);
                    account.gas.accumulate = u64::from_le_bytes(value[40..48].try_into()?);
                    account.gas.transfer = u64::from_le_bytes(value[48..56].try_into()?);
                }
                StateKey::Account {
                    service,
                    field: ServiceField::Storage,
                } => {
                    if service != index {
                        continue;
                    }

                    account.storage.insert(key.to_vec(), value);
                }
                StateKey::Account {
                    service,
                    field: ServiceField::Preimage,
                } => {
                    if service != index {
                        continue;
                    }

                    // TODO: verify the hash of the key
                    account.preimage.insert(crypto::blake2b(&value), value);
                }
                StateKey::Account {
                    service,
                    field: ServiceField::Lookup { length },
                } => {
                    if service != index {
                        continue;
                    }

                    let mut skey = [0; 32];
                    skey[..31].copy_from_slice(&key);
                    account
                        .lookup
                        .insert((skey, length), codec::decode(&value)?);
                }
                _ => continue,
            }
        }

        Ok(account)
    }

    /// Fetch the account state
    fn account_data(&self, service: u32) -> Result<ServiceData> {
        self.state_get(account::info(service))?
            .map(|value| codec::decode(&value))
            .ok_or(anyhow::anyhow!("account state not found"))?
            .map_err(|e| anyhow::anyhow!("failed to decode account state: {e}"))
    }

    /// Fetch the account storage
    fn account_storage(&self, service: u32, key: &[u8]) -> Result<Vec<u8>> {
        self.state_get(account::storage(service, key))?
            .map(|value| codec::decode(&value))
            .ok_or(anyhow::anyhow!("account storage not found"))?
            .map_err(|e| anyhow::anyhow!("failed to decode account storage: {e}"))
    }

    /// Fetch the account preimage
    fn account_preimage(&self, service: u32, key: OpaqueHash) -> Result<Vec<u8>> {
        self.state_get(account::preimage(service, key))?
            .map(|value| codec::decode(&value))
            .ok_or(anyhow::anyhow!("account preimage not found"))?
            .map_err(|e| anyhow::anyhow!("failed to decode account preimage: {e}"))
    }

    /// Fetch the account lookup
    fn account_lookup(&self, service: u32, lookup: u32, hash: OpaqueHash) -> Result<Vec<u32>> {
        self.state_get(account::lookup(service, lookup, hash))?
            .map(|value| codec::decode(&value))
            .ok_or(anyhow::anyhow!("account lookup not found"))?
            .map_err(|e| anyhow::anyhow!("failed to decode account lookup: {e}"))
    }

    /// Check if the storage contains the code
    fn contains_code(&self, code: OpaqueHash) -> Option<ServiceId> {
        for pair in self.state_prefix_iter(&[255]).ok()? {
            let (key, value) = pair.ok()?;
            if value.starts_with(&code) {
                return Some(u32::from_le_bytes([key[1], key[3], key[5], key[7]]));
            }
        }

        None
    }
}
