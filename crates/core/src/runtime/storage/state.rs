//! Storage APIs of the state of SpaceJam

use crate::{
    block::BlockInfo,
    extrinsic::DisputesRecords,
    runtime::storage::{branch, Branch, KVStorage},
    safrole::{Safrole, ValidatorData},
    service::{ServiceAccountState, ServiceIndex, WorkReport},
    state::{account, key, State},
    statistic::Statistics,
    EntropyBuffer, OpaqueHash, TimeSlot, CORES_COUNT, EPOCH_LENGTH,
};
use anyhow::{Context, Result};

/// Storage of the state of SpaceJam
///
/// the provided methods in the trait performs storage IO,
/// for higher performance, please reduce the number of IO operations
/// as much as possible.
pub trait Storage: KVStorage + Sized {
    /// Set the current branch of the storage
    ///
    /// TODO: support jump block, validate history blocks before
    /// switching to a new branch, if the target branch doesn not
    /// exists, we iter the history branch to find the parent of
    /// the target branch and checkout.
    fn checkout(&self, branch: OpaqueHash) -> Branch<Self> {
        Branch::new(self, branch)
    }

    /// Drop the target branch of the storage
    fn drop_branch(&mut self, branch: OpaqueHash) -> Result<()> {
        let mut prefix = branch::BRANCH_PREFIX.to_vec();
        prefix.extend_from_slice(&branch);

        while let Some(Ok((k, _))) = self.prefix_iter(&prefix)?.next() {
            self.remove(k)?;
        }
        Ok(())
    }

    /// Batch read a set of key-value pairs from the storage
    ///
    /// TODO: rename this method to something else since this for
    /// fetching jam specified prefix data only.
    fn prefix_collect(&self, prefix: [u8; 4]) -> Result<Vec<([u8; 32], Vec<u8>)>> {
        let mut kvs = vec![];
        let mut index = 0;
        loop {
            let mut storage_iter = self.prefix_iter(key::prefix(index, &prefix))?;
            let mut count = 0;
            while let Some(Ok((key, value))) = storage_iter.next() {
                let mut hkey = [0; 32];
                if key.len() == branch::MAIN_KEY_LENGTH {
                    hkey.copy_from_slice(&key);
                } else if key.len() == branch::BRANCH_KEY_LENGTH {
                    hkey.copy_from_slice(&key[6..38]);
                } else {
                    anyhow::bail!("invalid key length: {}", key.len());
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
        let data: Vec<Vec<u8>> = self
            .batch_read(vec![
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
            ])?
            .into_iter()
            .map(|(_, v)| v)
            .collect::<Vec<_>>();

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

        Ok(crypto::merkle::trie(&kvs, 0))
    }

    /// Finalize a branch
    ///
    /// A branch contains the diff of the state introduced in a block, so we need to
    /// apply it to the current state to get the final state.
    fn finalize(&self, branch: OpaqueHash) -> Result<()> {
        let mut prefix = branch::BRANCH_PREFIX.to_vec();
        prefix.extend_from_slice(&branch);

        // TODO: use transaction to reduce I/O & make this operations atomic.
        let mut storage_iter = self.prefix_iter(&prefix)?;
        while let Some(Ok((key, value))) = storage_iter.next() {
            if key.len() != 70 {
                anyhow::bail!("invalid key length: {}", key.len());
            }

            self.set(&key[38..], value)?;

            // TODO: maybe we can remove the diff after 15 blocks?
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
    fn timeslot(&self) -> Result<TimeSlot> {
        codec::decode(
            &self
                .get(key::TIMESLOT)?
                .ok_or(anyhow::anyhow!("timeslot not found"))?,
        )
        .context("failed to decode timeslot")
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
    fn account_info(&self, service: u32) -> Result<Option<ServiceAccountState>> {
        self.get(account::info(service))?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode account state: {e}"))
    }

    /// Fetch the account storage
    fn account_storage(&self, service: u32, key: OpaqueHash) -> Result<Option<Vec<u8>>> {
        self.get(account::storage(service, key))?
            .map(|value| codec::decode(&value))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to decode account storage: {e}"))
    }

    /// Fetch the account preimage
    fn account_preimage(&self, service: u32, key: OpaqueHash) -> Result<Option<Vec<u8>>> {
        self.get(account::preimage(service, key))?
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
