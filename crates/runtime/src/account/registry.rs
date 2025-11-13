//! Account registry with cached state

use crate::{Storage, account::Account};
use account::Account as _;
use score::OpaqueHash;
use std::{
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    sync::Arc,
};

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

impl<S: Storage> account::Accounts for Accounts<S> {
    fn blob(&mut self, index: u32) -> Option<Vec<u8>> {
        let account = self.get(index)?;
        let code = account.code();
        account.preimage(code)
    }

    fn get(&mut self, index: u32) -> Option<&mut impl ::account::Account> {
        if self.removed.contains(&index) {
            return None;
        }

        if let Entry::Vacant(e) = self.accounts.entry(index) {
            e.insert(Account::new(self.storage.clone(), index).ok()?);
        }

        self.accounts.get_mut(&index)
    }

    fn code_hash(&self, index: u32) -> Option<OpaqueHash> {
        if let Some(account) = self.accounts.get(&index) {
            return Some(account.info.code);
        }

        // WORKAROUND:
        //
        // always return the code hash from storage since this
        // might be updated during the execution, we can optimize
        // this later then.
        self.storage.account_info(index).ok().map(|info| info.code)
    }

    fn upsert(&mut self, index: u32, account: impl account::Account) {
        let inherited = Account::inherit(self.storage.clone(), index, account);
        self.accounts.insert(index, inherited);
    }

    fn remove(&mut self, index: u32) {
        self.accounts.remove(&index);
        self.removed.insert(index);
    }

    fn accounts(&self) -> &BTreeMap<u32, impl ::account::Account> {
        &self.accounts
    }

    fn removed(&self) -> BTreeSet<u32> {
        self.removed.clone()
    }

    fn diff(self) -> (Vec<([u8; 31], Vec<u8>)>, Vec<[u8; 31]>) {
        let mut updates = Vec::new();
        let mut removals = Vec::new();
        for (_, account) in self.accounts {
            let (lupdates, lremovals) = account.ops();
            updates.extend(lupdates);
            removals.extend(lremovals);
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
