use crate::{
    runtime::storage::{KVStorage, Storage},
    OpaqueHash,
};
use anyhow::Result;

/// The prefix of the branch key
pub const BRANCH_PREFIX: [u8; 6] = *b"branch";

/// The storage key length on main branch
pub const MAIN_KEY_LENGTH: usize = 32;

/// The storage key length on branch
pub const BRANCH_KEY_LENGTH: usize = 70;

/// A branch of the storage
pub struct Branch<'s, S: Storage> {
    storage: &'s S,
    branch: OpaqueHash,
}

impl<S: Storage> KVStorage for Branch<'_, S> {
    fn remove(&self, _key: impl AsRef<[u8]>) -> Result<()> {
        anyhow::bail!("remove is not supported on branch")
    }

    // TODO: ban set on branch
    fn set(&self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<()> {
        self.storage.set(self.key(key)?, value)
    }

    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        let key: &[u8] = key.as_ref();
        let Ok(Some(value)) = self.storage.get(self.key(key)?) else {
            return self.storage.get(key);
        };

        Ok(Some(value))
    }

    fn batch_write(&self, mut kvs: Vec<(Vec<u8>, Vec<u8>)>) -> Result<()> {
        for (k, _) in kvs.iter_mut() {
            *k = self.key(&k)?;
        }
        self.storage.batch_write(kvs)
    }

    fn batch_read(&self, keys: Vec<OpaqueHash>) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let mut kvs = self.storage.batch_read(keys)?;
        for (k, v) in kvs.iter_mut() {
            if let Ok(Some(value)) = self.get(k) {
                *v = value;
            }
        }
        Ok(kvs)
    }

    fn prefix_iter(
        &self,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        self.storage.prefix_iter(self.key(prefix)?)
    }
}

impl<'s, S: Storage> Branch<'s, S> {
    /// Create a new branch
    pub fn new(storage: &'s S, branch: OpaqueHash) -> Self {
        Self { storage, branch }
    }

    /// Initialize a diff of the storage
    pub fn diff(&self) -> Diff {
        Diff::default()
    }

    /// Commit the diff to the storage
    ///
    /// The instance should be dropped after commitment.
    pub fn commit(self, diff: Diff) -> Result<()> {
        self.storage.batch_write(diff.collect())
    }

    /// Get the branch key
    ///
    /// This interface also supports branch of branch.
    fn key(&self, key: impl AsRef<[u8]>) -> Result<Vec<u8>> {
        let key = key.as_ref();
        if key.len() == BRANCH_KEY_LENGTH {
            let mut bkey = key.to_vec();
            bkey[6..38].copy_from_slice(&self.branch);
            return Ok(bkey);
        }

        if !key.len() == MAIN_KEY_LENGTH {
            anyhow::bail!("invalid key length in branch: {}", key.len());
        }

        let mut bkey = [0; BRANCH_KEY_LENGTH];
        bkey[..6].copy_from_slice(&BRANCH_PREFIX);
        bkey[6..38].copy_from_slice(&self.branch);
        bkey[38..].copy_from_slice(key.as_ref());
        Ok(bkey.to_vec())
    }
}

/// A diff of the state
#[derive(Default)]
pub struct Diff {
    diff: Vec<(Vec<u8>, Vec<u8>)>,
}

impl Diff {
    /// Insert a key-value pair into the diff
    pub fn insert(&mut self, key: OpaqueHash, value: impl AsRef<[u8]>) {
        self.diff.push((key.to_vec(), value.as_ref().to_vec()));
    }

    /// Extend the diff with a set of key-value pairs
    pub fn extend(&mut self, kvs: Vec<(OpaqueHash, Vec<u8>)>) {
        self.diff
            .extend(kvs.into_iter().map(|(key, value)| (key.to_vec(), value)));
    }

    /// Collect the diff
    pub fn collect(self) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.diff
    }
}
