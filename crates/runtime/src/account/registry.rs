//! Account registry with cached state

use crate::{account::Account, Storage};
use score::{state, Account as _, OpaqueHash};
use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    sync::Arc,
};
use tokio::task::JoinSet;

/// Account registry with cached state
pub struct Accounts<S: Storage> {
    /// The storage of the accounts
    storage: Arc<S>,

    /// The account registry
    accounts: BTreeMap<u32, Account<S>>,

    /// The removed accounts
    removed: BTreeSet<u32>,
}

impl<S: Storage> Accounts<S> {
    /// Create a new account registry
    ///
    /// TODO: fetch initial accounts based on input extrinsics
    pub fn new(storage: Arc<S>) -> Self {
        Self {
            storage,
            accounts: BTreeMap::new(),
            removed: BTreeSet::new(),
        }
    }
}

impl<S: Storage> score::Accounts for Accounts<S> {
    fn blob(&mut self, index: u32) -> Option<Vec<u8>> {
        let account = self.get(index)?;
        let code = account.code();
        self.storage
            .state_get(state::account::preimage(index, code))
            .ok()?
            .map(|v| v.to_vec())
    }

    fn get(&mut self, index: u32) -> Option<&mut impl score::Account> {
        if let Entry::Vacant(e) = self.accounts.entry(index) {
            e.insert(Account::new(self.storage.clone(), index).ok()?);
        }

        self.accounts.get_mut(&index)
    }

    fn code_hash(&self, index: u32) -> Option<OpaqueHash> {
        // WORKAROUND:
        //
        // always return the code hash from storage since this
        // might be updated during the execution, we can optimize
        // this later then.
        self.storage.account_info(index).ok().map(|info| info.code)
    }

    fn upsert(&mut self, index: u32, account: impl score::Account) {
        let inherited = Account::inherit(self.storage.clone(), index, account);
        self.accounts.insert(index, inherited);
    }

    fn remove(&mut self, index: u32) {
        self.accounts.remove(&index);
        self.removed.insert(index);
    }

    fn services(&self) -> Vec<u32> {
        self.accounts.keys().cloned().collect()
    }

    fn accounts(&self) -> &BTreeMap<u32, impl score::Account> {
        &self.accounts
    }

    fn removed(&self) -> BTreeSet<u32> {
        self.removed.clone()
    }

    async fn diff(self) -> (Vec<([u8; 31], Vec<u8>)>, Vec<[u8; 31]>) {
        let mut updates = Vec::new();
        let mut removals = Vec::new();

        let mut all_updates = Vec::new();
        let mut all_removals = Vec::new();
        for (_, account) in self.accounts {
            let (aupdates, aremovals) = account.updates();
            all_updates.extend(aupdates);
            all_removals.extend(aremovals);
        }

        let mut set_updates: JoinSet<_> = all_updates
            .into_iter()
            .map(|(k, v)| async move { (k.trie(), v) })
            .collect();
        let mut set_removals: JoinSet<_> = all_removals
            .into_iter()
            .map(|k| async move { k.trie() })
            .collect();
        while let Some(result) = set_updates.join_next().await {
            updates.push(result.unwrap_or_default());
        }
        while let Some(result) = set_removals.join_next().await {
            removals.push(result.unwrap_or_default());
        }

        for index in self.removed {
            let keys = self.storage.account_keys(index).unwrap_or_default();
            removals.extend(keys);
        }

        (updates, removals)
    }
}

impl<S: Storage> Clone for Accounts<S> {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            accounts: self.accounts.clone(),
            removed: self.removed.clone(),
        }
    }
}
