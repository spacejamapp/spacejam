//! Storage APIs of the state of SpaceJam
//!
//! TODO: tests for IO. e.g. encoding / decoding of kvs.

use crate::storage::KVStorage;
use anyhow::{Context, Result};
use score::{
    CORES_COUNT, EPOCH_LENGTH, EntropyBuffer, OpaqueHash, TimeSlot,
    block::BlockInfo,
    extrinsic::DisputesRecords,
    safrole::{Safrole, ValidatorData},
    service::{
        Privileges, ServiceAccountData, ServiceAccountState, ServiceItem, ServicePreimage,
        WorkReport,
    },
    state::{State, account, key},
    statistic::Statistics,
};

/// Storage of the state of SpaceJam
///
/// the provided methods in the trait performs storage IO,
/// for higher performance, please reduce the number of IO operations
/// as much as possible.
pub trait Storage: KVStorage {
    /// Batch read a set of key-value pairs from the storage
    ///
    /// TODO: rename this method to something else since this for
    /// fetching jam specified prefix data only.
    fn prefix_collect(&self, prefix: [u8; 4]) -> Result<Vec<([u8; 32], Vec<u8>)>> {
        let mut kvs = vec![];
        let mut index = 0;
        loop {
            let Ok(mut storage_iter) = self.prefix_iter(key::prefix(index, &prefix)) else {
                break;
            };

            while let Some(Ok((key, value))) = storage_iter.next() {
                let mut hkey = [0; 32];
                hkey.copy_from_slice(&key);
                kvs.push((hkey, value));
            }

            index += 1;
        }

        Ok(kvs)
    }

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

        // we don't need to batch all state in the memory to calculate the root since we can use
        // the prefix of storage keys to iterate them.
        //
        // We need to read the state for validating blocks.
        Ok(state)
    }

    /// Calculate the root of the state from storage.
    fn root(&self) -> Result<OpaqueHash> {
        let mut kvs = vec![];
        for key in key::CONSTANT_KEYS {
            kvs.push((key, self.get(key)?.unwrap_or_default()));
        }

        // fetch account state
        let mut service = 0;
        while let Some(state) = self.get(account::info(service))? {
            kvs.push((account::info(service), state));
            service += 1;
        }

        // fetch account storage and preimage
        for prefix in [key::ACCOUNT_STORAGE_PREFIX, key::ACCOUNT_PREIMAGE_PREFIX] {
            kvs.extend(self.prefix_collect(prefix)?);
        }

        // fetch lookup data
        //
        // TODO: double check how to iterate the lookup data
        let mut service: u32 = 0;
        while let Ok(lookup) = self.prefix_collect(service.to_le_bytes()) {
            kvs.extend(lookup);
            service += 1;
        }

        Ok(crypto::merkle::trie(&kvs, 0))
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
    fn next_validators(&self) -> Result<Vec<ValidatorData>> {
        codec::decode(
            &self
                .get(key::NEXT_VALIDATORS)?
                .ok_or(anyhow::anyhow!("next validators not found"))?,
        )
        .context("failed to decode next validators")
    }

    /// Fetch the current validators
    fn current_validators(&self) -> Result<Vec<ValidatorData>> {
        codec::decode(
            &self
                .get(key::CURRENT_VALIDATORS)?
                .ok_or(anyhow::anyhow!("current validators not found"))?,
        )
        .context("failed to decode current validators")
    }

    /// Fetch the previous validators
    fn previous_validators(&self) -> Result<Vec<ValidatorData>> {
        codec::decode(
            &self
                .get(key::PREVIOUS_VALIDATORS)?
                .ok_or(anyhow::anyhow!("previous validators not found"))?,
        )
        .context("failed to decode previous validators")
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
        Ok(ServiceItem {
            id: service,
            data: ServiceAccountData {
                service: info,
                preimages,
            },
        })
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

    fn account_preimages(&self, service: u32) -> Result<Vec<ServicePreimage>> {
        self.prefix_iter(key::prefix(service, &key::ACCOUNT_PREIMAGE_PREFIX))?
            .map(|kv| {
                kv.map(|(key, value)| {
                    let mut hash = [0; 32];
                    hash.copy_from_slice(&key);
                    ServicePreimage { hash, blob: value }
                })
            })
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

    /// Set the service account info
    fn set_info(&self, service: u32, acc: &ServiceAccountState) -> Result<()> {
        let mut value = Vec::new();
        value.extend_from_slice(&acc.code);
        value.extend_from_slice(&codec::encode(&(
            &acc.balance,
            &acc.gas.accumulate,
            &acc.gas.transfer,
            &acc.total,
        ))?);
        value.extend_from_slice(&acc.items.to_le_bytes());
        self.set(account::info(service), value)
    }

    /// Set the service account storage
    fn set_storage(&self, service: u32, key: OpaqueHash, value: impl AsRef<[u8]>) -> Result<()> {
        self.set(account::storage(service, key), value)
    }

    /// Set the service account preimage
    fn set_preimage(&self, service: u32, key: OpaqueHash, value: impl AsRef<[u8]>) -> Result<()> {
        self.set(account::preimage(service, key), value)
    }

    /// Set the service account lookup
    fn set_lookup(
        &self,
        service: u32,
        lookup: u32,
        key: OpaqueHash,
        slots: [TimeSlot; 3],
    ) -> Result<()> {
        self.set(
            account::lookup(service, lookup, key),
            codec::encode(&slots)?,
        )
    }
}
