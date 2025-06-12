//! Storage APIs of the state of SpaceJam
//!
//! TODO: tests for IO. e.g. encoding / decoding of kvs.

use std::collections::BTreeMap;

use crate::storage::{KVStorage, sync};
use anyhow::{Context, Result};
use crypto::merkle;
use score::{
    CORES_COUNT, EPOCH_LENGTH, EntropyBuffer, OpaqueHash, TimeSlot,
    block::BlockInfo,
    extrinsic::DisputesRecords,
    safrole::{Safrole, ValidatorsData},
    service::{
        AvailabilityAssignments, Privileges, ServiceAccount, ServiceAccountData,
        ServiceAccountState, ServiceItem, ServicePreimage, ServiceStorage, WorkReport,
    },
    state::{ServiceField, State, StateKey, StateKeyInfo, StateKeyLike, account, key},
    statistic::Statistics,
};

/// Storage of the state of SpaceJam
///
/// the provided methods in the trait performs storage IO,
/// for higher performance, please reduce the number of IO operations
/// as much as possible.
pub trait Storage: KVStorage {
    /// Fetch state from the storage
    ///
    /// We don't decode account data in this batch since it will be too large.
    fn state(&self) -> Result<State> {
        let mut state = State::default();
        let data: Vec<Vec<u8>> = self
            .batch_read(
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

        state.pools = codec::decode(data.first().unwrap_or(&vec![])).unwrap_or_default();
        state.authorization = codec::decode(data.get(1).unwrap_or(&vec![])).unwrap_or_default();
        state.recent_blocks = codec::decode(data.get(2).unwrap_or(&vec![])).unwrap_or_default();
        state.safrole = codec::decode(data.get(3).unwrap_or(&vec![])).unwrap_or_default();
        state.disputes = codec::decode(data.get(4).unwrap_or(&vec![])).unwrap_or_default();
        state.entropy = codec::decode(data.get(5).unwrap_or(&vec![])).unwrap_or_default();
        state.validators.next = codec::decode(data.get(6).unwrap_or(&vec![])).unwrap_or_default();
        state.validators.current =
            codec::decode(data.get(7).unwrap_or(&vec![])).unwrap_or_default();
        state.validators.previous =
            codec::decode(data.get(8).unwrap_or(&vec![])).unwrap_or_default();
        state.reports = codec::decode(data.get(9).unwrap_or(&vec![])).unwrap_or_default();
        state.timeslot = codec::decode(data.get(10).unwrap_or(&vec![])).unwrap_or_default();
        state.privileges = codec::decode(data.get(11).unwrap_or(&vec![])).unwrap_or_default();
        state.statistics = codec::decode(data.get(12).unwrap_or(&vec![])).unwrap_or_default();
        state.queue = codec::decode(data.get(13).unwrap_or(&vec![])).unwrap_or_default();
        state.history = codec::decode(data.get(14).unwrap_or(&vec![])).unwrap_or_default();
        state.accounts = self.accounts()?;

        // we don't need to batch all state in the memory to calculate the root since we can use
        // the prefix of storage keys to iterate them.
        //
        // We need to read the state for validating blocks.
        Ok(state)
    }

    /// Calculate the root of the state from storage.
    ///
    /// FIXME: it is not ideal to store all data in memory
    /// for calculating the root.
    fn root(&self) -> Result<OpaqueHash> {
        let mut kvs = Vec::new();
        for pair in self.iter()? {
            let (k, v) = pair?;
            if k.starts_with(sync::BLOCK_KEY) || k.starts_with(sync::SERIES_KEY) || k.len() != 31 {
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
    fn recent_blocks(&self) -> Result<Vec<BlockInfo>> {
        codec::decode(
            &self
                .get(key::RECENT_BLOCKS)?
                .ok_or(anyhow::anyhow!("recent blocks not found"))?,
        )
        .context("failed to decode recent blocks")
    }

    /// Fetch the safrole state
    fn safrole(&self) -> Result<Safrole> {
        codec::decode(
            &self
                .get(key::SAFROLE)?
                .ok_or(anyhow::anyhow!("safrole not found"))?,
        )
        .context("failed to decode safrole")
    }

    /// Fetch the judgements from the storage
    fn disputes(&self) -> Result<Option<DisputesRecords>> {
        self.get(key::DISPUTES)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode disputes: {e}"))
    }

    /// Fetch the entropy state
    fn entropy(&self) -> Result<EntropyBuffer> {
        codec::decode(
            &self
                .get(key::ENTROPY)?
                .ok_or(anyhow::anyhow!("entropy not found"))?,
        )
        .context("failed to decode entropy")
    }

    /// Fetch the next validators
    fn next_validators(&self) -> Result<ValidatorsData> {
        codec::decode(
            &self
                .get(key::NEXT_VALIDATORS)?
                .ok_or(anyhow::anyhow!("next validators not found"))?,
        )
        .context("failed to decode next validators")
    }

    /// Fetch the current validators
    fn current_validators(&self) -> Result<ValidatorsData> {
        codec::decode(
            &self
                .get(key::CURRENT_VALIDATORS)?
                .ok_or(anyhow::anyhow!("current validators not found"))?,
        )
        .context("failed to decode current validators")
    }

    /// Fetch the previous validators
    fn previous_validators(&self) -> Result<ValidatorsData> {
        codec::decode(
            &self
                .get(key::PREVIOUS_VALIDATORS)?
                .ok_or(anyhow::anyhow!("previous validators not found"))?,
        )
        .context("failed to decode previous validators")
    }

    /// Fetch the pending reports
    fn pending_reports(&self) -> Result<AvailabilityAssignments> {
        self.get(key::PENDING_REPORTS)?
            .map(|value| codec::decode(&value))
            .ok_or(anyhow::anyhow!("pending reports not found"))?
            .map_err(|e| anyhow::anyhow!("failed to decode pending reports: {e}"))
    }

    /// Fetch the timeslot
    fn timeslot(&self) -> Result<TimeSlot> {
        codec::decode(
            &self
                .get(key::TIMESLOT)?
                .ok_or(anyhow::anyhow!("timeslot not found"))?,
        )
        .context("failed to decode timeslot")
    }

    /// Fetch the privileged service indices
    fn privileges(&self) -> Result<Privileges> {
        self.get(key::PRIVILEGED_SERVICE)?
            .map(|value| codec::decode(&value))
            .ok_or(anyhow::anyhow!("privileged service not found"))?
            .map_err(|e| anyhow::anyhow!("failed to decode privileged service: {e}"))
    }

    /// Fetch the activity statistics
    fn statistics(&self) -> Result<Statistics> {
        self.get(key::STATISTICS)?
            .map(|value| codec::decode(&value))
            .ok_or(anyhow::anyhow!("statistics not found"))?
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

    /// Fetch the account
    fn account(&self, service: u32) -> Result<ServiceItem> {
        let info = self.account_info(service)?;
        let preimages = self.account_preimages(service)?;
        let storage = self.account_storage_full(service)?;
        Ok(ServiceItem {
            id: service,
            data: ServiceAccountData {
                service: info,
                preimages,
                storage,
            },
        })
    }

    /// FIXME: do not use this method in production
    fn accounts(&self) -> Result<BTreeMap<u32, ServiceAccount>> {
        let mut accounts = BTreeMap::new();
        for item in self.iter()? {
            let (key, value) = item?;
            match key.as_state_key().info() {
                StateKey::Account {
                    service,
                    field: ServiceField::Data,
                } => {
                    let account: &mut ServiceAccount = accounts.entry(service).or_default();
                    account.code = value[..32].try_into()?;
                    account.balance = u64::from_le_bytes(value[32..40].try_into()?);
                    account.gas.accumulate = u64::from_le_bytes(value[40..48].try_into()?);
                    account.gas.transfer = u64::from_le_bytes(value[48..56].try_into()?);
                }
                StateKey::Account {
                    service,
                    field: ServiceField::Storage,
                } => {
                    let account: &mut ServiceAccount = accounts.entry(service).or_default();
                    account.storage.insert(key.to_vec(), value);
                }
                StateKey::Account {
                    service,
                    field: ServiceField::Preimage,
                } => {
                    // TODO: verify the hash of the key
                    let account: &mut ServiceAccount = accounts.entry(service).or_default();
                    account.preimage.insert(crypto::blake2b(&value), value);
                }
                StateKey::Account {
                    service,
                    field: ServiceField::Lookup { length },
                } => {
                    let account: &mut ServiceAccount = accounts.entry(service).or_default();
                    account
                        .lookup
                        .insert((Default::default(), length), codec::decode(&value)?);
                }
                _ => continue,
            }
        }

        // FIXME: this is a temporary solution to fill the lookup field
        //
        // TODO: shared preimages, consider zero-copy solution
        for (_, account) in accounts.iter_mut() {
            let lookup = account.lookup.clone();
            for (key, value) in lookup.iter() {
                for (hash, preimage) in &account.preimage {
                    if key.1 == preimage.len() as u32 {
                        account.lookup.insert((*hash, key.1), value.clone());
                        account.lookup.remove(key);
                    }
                }
            }
        }

        Ok(accounts)
    }

    /// Fetch the account state
    fn account_info(&self, service: u32) -> Result<ServiceAccountState> {
        self.get(account::info(service))?
            .map(|value| codec::decode(&value))
            .ok_or(anyhow::anyhow!("account state not found"))?
            .map_err(|e| anyhow::anyhow!("failed to decode account state: {e}"))
    }

    /// Fetch the account storage
    fn account_storage(&self, service: u32, key: OpaqueHash) -> Result<Vec<u8>> {
        self.get(account::storage(service, key))?
            .map(|value| codec::decode(&value))
            .ok_or(anyhow::anyhow!("account storage not found"))?
            .map_err(|e| anyhow::anyhow!("failed to decode account storage: {e}"))
    }

    /// Fetch the account preimage
    fn account_preimage(&self, service: u32, key: OpaqueHash) -> Result<Vec<u8>> {
        self.get(account::preimage(service, key))?
            .map(|value| codec::decode(&value))
            .ok_or(anyhow::anyhow!("account preimage not found"))?
            .map_err(|e| anyhow::anyhow!("failed to decode account preimage: {e}"))
    }

    /// Fetch the account preimages
    fn account_preimages(&self, service: u32) -> Result<Vec<ServicePreimage>> {
        self.prefix_iter(key::prefix(service, &key::ACCOUNT_PREIMAGE_PREFIX))?
            .map(|kv| {
                kv.map(|(_, value)| {
                    // FIXME: cache the preimage somewhere else
                    let hash = crypto::blake2b(&value);
                    ServicePreimage { hash, blob: value }
                })
            })
            .collect()
    }

    fn account_storage_full(&self, service: u32) -> Result<Vec<ServiceStorage>> {
        self.prefix_iter(key::prefix(service, &key::ACCOUNT_STORAGE_PREFIX))?
            .map(|kv| kv.map(|(key, value)| ServiceStorage { key, value }))
            .collect()
    }

    /// Fetch the account lookup
    fn account_lookup(
        &self,
        service: u32,
        lookup: u32,
        key: OpaqueHash,
    ) -> Result<Option<[TimeSlot; 3]>> {
        self.get(account::lookup(service, lookup, key))?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode account lookup: {e}"))
    }
}
