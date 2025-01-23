//! Storage APIs of the state of SpaceJam

use crate::{
    block::BlockInfo,
    extrinsic::DisputesRecords,
    safrole::Safrole,
    service::{ServiceAccountState, ServiceIndex},
    state::{account, key, State},
    statistic::Statistics,
    validator::ValidatorData,
    work::report::WorkReport,
    EntropyBuffer, OpaqueHash, TimeSlot, CORES_COUNT, EPOCH_LENGTH,
};
use anyhow::Result;
use std::path::Path;

/// The prefix of the branch key
const BRANCH_PREFIX: [u8; 6] = *b"branch";

/// Storage of the state of SpaceJam
///
/// the provided methods in the trait performs storage IO,
/// for higher performance, please reduce the number of IO operations
/// as much as possible.
pub trait Storage: Sized {
    /// Open the storage from path
    fn open(path: impl AsRef<Path>) -> Result<Self>;

    /// Get the current branch of the storage
    fn branch(&self) -> Option<OpaqueHash>;

    /// Set the current branch of the storage
    fn checkout(&mut self, _branch: Option<OpaqueHash>);

    /// Set a value in the storage
    fn set(&self, _key: impl AsRef<[u8]>, _value: impl AsRef<[u8]>) -> Result<()>;

    /// Batch write a set of key-value pairs to the storage
    fn batch_write(&self, kvs: Vec<(Vec<u8>, Vec<u8>)>) -> Result<()>;

    /// Get a value from the storage
    fn get(&self, _key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>>;

    /// Batch read a set of key-value pairs from the storage
    fn batch_read(&self, keys: Vec<Vec<u8>>) -> Result<Vec<Vec<u8>>>;

    /// Remove a key-value pair from the storage
    fn remove(&self, key: impl AsRef<[u8]>) -> Result<()>;

    /// Iterate over the storage with a prefix
    fn prefix_iter(
        &self,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>>;

    /// Check if the storage is empty
    fn is_empty(&self) -> bool {
        self.get(key::TIMESLOT).map(|v| v.is_none()).unwrap_or(true)
    }

    /// Get the branch key
    fn branch_key(&self, branch: OpaqueHash, key: OpaqueHash) -> [u8; 70] {
        let mut bkey = [0; 70];
        bkey[0..6].copy_from_slice(&BRANCH_PREFIX);
        bkey[6..38].copy_from_slice(&branch[..32]);
        bkey[38..70].copy_from_slice(&key[..32]);
        bkey
    }

    /// Get the value of the branch
    fn branch_get(&self, key: [u8; 32]) -> Result<Option<Vec<u8>>> {
        let Some(bkey) = self.wrap_key(key) else {
            return Ok(None);
        };

        self.get(bkey)
    }

    /// Drop the target branch of the storage
    fn drop_branch(&mut self, branch: OpaqueHash) -> Result<()> {
        let mut prefix = BRANCH_PREFIX.to_vec();
        prefix.extend_from_slice(&branch[..8]);

        while let Some(Ok((k, _))) = self.prefix_iter(&prefix)?.next() {
            self.remove(k)?;
        }
        Ok(())
    }

    /// Wrap the key to the branch
    fn wrap_key(&self, key: [u8; 32]) -> Option<[u8; 70]> {
        let branch = self.branch()?;
        Some(self.branch_key(branch, key))
    }

    /// Wrap the key to the branch and get the value
    fn wrap_get(&self, key: [u8; 32]) -> Result<Option<Vec<u8>>> {
        if let Some(bkey) = self.wrap_key(key) {
            if let Ok(Some(result)) = self.get(bkey) {
                return Ok(Some(result));
            }
        }

        self.get(key)
    }

    /// Wrap the key to the branch and set the value
    fn wrap_set(&self, key: [u8; 32], value: impl AsRef<[u8]>) -> Result<()> {
        if let Some(bkey) = self.wrap_key(key) {
            self.set(bkey, value)
        } else {
            self.set(key, value)
        }
    }

    /// Batch read a set of key-value pairs from the storage
    fn prefix_collect(&self, prefix: [u8; 4]) -> Result<Vec<([u8; 32], Vec<u8>)>> {
        let mut kvs = vec![];
        let mut index = 0;
        loop {
            let mut storage_iter = self.prefix_iter(key::prefix(index, &prefix))?;
            let mut count = 0;
            while let Some(Ok((key, mut value))) = storage_iter.next() {
                let hkey = OpaqueHash::try_from(key)
                    .map_err(|e| anyhow::anyhow!("invalid prefix key: {e:?}"))?;
                if let Some(bvalue) = self.branch_get(hkey)? {
                    value = bvalue;
                }

                kvs.push((hkey, value));
                count += 1;
            }

            if count == 0 {
                break;
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
        let data: Vec<Vec<u8>> = self.batch_read(
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
        )?;

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
        let mut service: u32 = 0;
        while let Ok(lookup) = self.prefix_collect(service.to_le_bytes()) {
            kvs.extend(lookup);
            service += 1;
        }

        Ok(merkle::trie(&kvs, 0))
    }

    /// Stash the state to the branch
    fn stash(&self, branch: OpaqueHash, diff: Vec<(OpaqueHash, Vec<u8>)>) -> Result<()> {
        let batch = diff
            .into_iter()
            .map(|(key, value)| (self.branch_key(branch, key).to_vec(), value))
            .collect::<Vec<_>>();
        self.batch_write(batch)?;
        Ok(())
    }

    /// Finalize a branch
    ///
    /// A branch contains the diff of the state introduced in a block, so we need to
    /// apply it to the current state to get the final state.
    fn finalize(&self, branch: OpaqueHash) -> Result<()> {
        let prefix = &self.branch_key(branch, Default::default())[..38];
        let mut storage_iter = self.prefix_iter(prefix)?;

        // TODO: use transaction to reduce I/O & make this operations atomic.
        while let Some(Ok((key, value))) = storage_iter.next() {
            self.set(&key, value)?;
            self.remove(&key)?;
        }
        Ok(())
    }

    /// Fetch the authorization pools from the storage
    fn pools(&self) -> Result<Option<[Vec<OpaqueHash>; CORES_COUNT]>> {
        self.wrap_get(key::AUTHORIZATION_POOLS)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode pools: {e}"))
    }

    /// Fetch the authorization queue from the storage
    fn authorization_queue(&self) -> Result<Option<[Vec<OpaqueHash>; CORES_COUNT]>> {
        self.wrap_get(key::AUTHORIZATION_QUEUE)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode authorization queue: {e}"))
    }

    /// Fetch the recent blocks from the storage
    fn recent_blocks(&self) -> Result<Option<Vec<BlockInfo>>> {
        self.wrap_get(key::RECENT_BLOCKS)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode recent blocks: {e}"))
    }

    /// Fetch the safrole state
    fn safrole(&self) -> Result<Option<Safrole>> {
        self.wrap_get(key::SAFROLE)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode safrole: {e}"))
    }

    /// Fetch the judgements from the storage
    fn disputes(&self) -> Result<Option<DisputesRecords>> {
        self.wrap_get(key::DISPUTES)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode disputes: {e}"))
    }

    /// Fetch the entropy state
    fn entropy(&self) -> Result<Option<EntropyBuffer>> {
        self.wrap_get(key::ENTROPY)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode entropy: {e}"))
    }

    /// Fetch the next validators
    fn next_validators(&self) -> Result<Option<Vec<ValidatorData>>> {
        self.wrap_get(key::NEXT_VALIDATORS)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode next validators: {e}"))
    }

    /// Fetch the current validators
    fn current_validators(&self) -> Result<Option<Vec<ValidatorData>>> {
        self.wrap_get(key::CURRENT_VALIDATORS)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode current validators: {e}"))
    }

    /// Fetch the previous validators
    fn previous_validators(&self) -> Result<Option<Vec<ValidatorData>>> {
        self.wrap_get(key::PREVIOUS_VALIDATORS)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode previous validators: {e}"))
    }

    /// Fetch the pending reports
    #[allow(clippy::type_complexity)]
    fn pending_reports(&self) -> Result<Option<[Option<(WorkReport, TimeSlot)>; CORES_COUNT]>> {
        self.wrap_get(key::PENDING_REPORTS)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode pending reports: {e}"))
    }

    /// Fetch the timeslot
    fn timeslot(&self) -> Result<Option<TimeSlot>> {
        self.wrap_get(key::TIMESLOT)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode timeslot: {e}"))
    }

    /// Fetch the privileged service indices
    fn service(&self) -> Result<Option<ServiceIndex>> {
        self.wrap_get(key::PRIVILEGED_SERVICE)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode privileged service: {e}"))
    }

    /// Fetch the activity statistics
    fn statistics(&self) -> Result<Option<Statistics>> {
        self.wrap_get(key::STATISTICS)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode statistics: {e}"))
    }

    /// Fetch the accumulation queue
    #[allow(clippy::type_complexity)]
    fn accumulation_queue(
        &self,
    ) -> Result<Option<[(Vec<WorkReport>, Vec<OpaqueHash>); EPOCH_LENGTH as usize]>> {
        self.wrap_get(key::ACCUMULATION_QUEUE)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode accumulation queue: {e}"))
    }

    /// Fetch the accumulation history
    fn accumulation_history(&self) -> Result<Option<[Vec<OpaqueHash>; EPOCH_LENGTH as usize]>> {
        self.wrap_get(key::ACCUMULATION_HISTORY)?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode accumulation history: {e}"))
    }

    /// Fetch the account state
    fn account_info(&self, service: u32) -> Result<Option<ServiceAccountState>> {
        self.wrap_get(account::info(service))?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode account state: {e}"))
    }

    /// Fetch the account storage
    fn account_storage(&self, service: u32, key: OpaqueHash) -> Result<Option<Vec<u8>>> {
        self.wrap_get(account::storage(service, key))?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode account storage: {e}"))
    }

    /// Fetch the account preimage
    fn account_preimage(&self, service: u32, key: OpaqueHash) -> Result<Option<Vec<u8>>> {
        self.wrap_get(account::preimage(service, key))?
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
        self.wrap_get(account::lookup(service, lookup, key))?
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
        self.wrap_set(account::info(service), value)
    }

    /// Set the service account storage
    fn set_storage(&self, service: u32, key: OpaqueHash, value: impl AsRef<[u8]>) -> Result<()> {
        self.wrap_set(account::storage(service, key), value)
    }

    /// Set the service account preimage
    fn set_preimage(&self, service: u32, key: OpaqueHash, value: impl AsRef<[u8]>) -> Result<()> {
        self.wrap_set(account::preimage(service, key), value)
    }

    /// Set the service account lookup
    fn set_lookup(
        &self,
        service: u32,
        lookup: u32,
        key: OpaqueHash,
        slots: [TimeSlot; 3],
    ) -> Result<()> {
        self.wrap_set(
            account::lookup(service, lookup, key),
            slots
                .iter()
                .flat_map(|slot| slot.to_le_bytes())
                .collect::<Vec<u8>>(),
        )
    }
}
