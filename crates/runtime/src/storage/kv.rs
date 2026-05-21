//! Key-value storage abstraction

use crate::storage::{Column, Commit, MultiTreeStore, NewNode, NodeAddress, NodeRef};
use anyhow::Result;
use score::{OpaqueHash, TrieKey, state::StateKeyLike};
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, RwLock},
};

/// Key-value storage
pub trait KVStorage: Send + Sync + 'static {
    /// Batch write a set of key-value pairs to the storage
    fn commit(&self, column: Column, commit: Commit<TrieKey, Vec<u8>>) -> Result<()>;

    /// Set a key-value pair with column specified
    fn set(&self, column: Column, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<()>;

    /// Get a value from the storage with column specified
    fn get(&self, column: Column, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>>;

    /// Iterate over the storage with column specified
    fn iter(&self, column: Column) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>>;

    /// Iterate over the storage with a prefix and column specified
    fn prefix_iter(
        &self,
        column: Column,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>>;

    /// Batch read a set of key-value pairs from the storage with column specified
    fn batch_read(&self, column: Column, keys: Vec<Vec<u8>>) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        keys.iter()
            .map(|key| {
                self.get(column, key)
                    .map(|v| (key.to_vec(), v.unwrap_or_default()))
            })
            .collect::<Result<Vec<_>>>()
    }
}

/// In-memory key-value storage implementation
///
/// This implementation stores all data in memory and is not persistent.
/// It's useful for testing and for situations where persistence isn't required.
#[derive(Default)]
pub struct MemoryDb {
    data: Arc<RwLock<BTreeMap<TrieKey, Vec<u8>>>>,
    tries: Arc<RwLock<TrieStore>>,
}

/// Internal ref-counted node store backing [`MultiTreeStore`].
#[derive(Default)]
struct TrieStore {
    nodes: HashMap<NodeAddress, NodeEntry>,
    roots: HashMap<OpaqueHash, NodeAddress>,
    next_addr: NodeAddress,
}

struct NodeEntry {
    data: Vec<u8>,
    children: Vec<NodeAddress>,
    refcount: u32,
}

impl TrieStore {
    fn alloc(&mut self) -> NodeAddress {
        let addr = self.next_addr;
        self.next_addr += 1;
        addr
    }

    /// Recursively materialize a `NodeRef` into the store, returning the
    /// address of its root and bumping refcounts as we go.
    fn insert(&mut self, node_ref: NodeRef) -> NodeAddress {
        match node_ref {
            NodeRef::Existing(addr) => {
                let entry = self
                    .nodes
                    .get_mut(&addr)
                    .unwrap_or_else(|| panic!("NodeRef::Existing({addr}) but node is gone"));
                entry.refcount += 1;
                addr
            }
            NodeRef::New(NewNode { data, children }) => {
                let child_addrs: Vec<_> = children.into_iter().map(|c| self.insert(c)).collect();
                let addr = self.alloc();
                self.nodes.insert(
                    addr,
                    NodeEntry {
                        data,
                        children: child_addrs,
                        refcount: 1,
                    },
                );
                addr
            }
        }
    }

    /// Drop a reference to `addr`, recursively GC'ing descendants that hit 0.
    fn dereference(&mut self, addr: NodeAddress) {
        let drop_children = match self.nodes.get_mut(&addr) {
            Some(entry) => {
                entry.refcount = entry
                    .refcount
                    .checked_sub(1)
                    .unwrap_or_else(|| panic!("refcount underflow on addr {addr}"));
                if entry.refcount == 0 {
                    Some(std::mem::take(&mut entry.children))
                } else {
                    None
                }
            }
            None => None,
        };
        if let Some(children) = drop_children {
            self.nodes.remove(&addr);
            for child in children {
                self.dereference(child);
            }
        }
    }
}

impl MemoryDb {
    /// Execute a closure with direct read access to the underlying data,
    /// holding the read lock for the duration.
    pub fn with_data<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&BTreeMap<TrieKey, Vec<u8>>) -> R,
    {
        let data = self
            .data
            .read()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;
        Ok(f(&data))
    }

    /// Reset the memory database
    pub fn reset(&self, data: BTreeMap<TrieKey, Vec<u8>>) {
        let mut curr = self.data.write().unwrap();
        *curr = data;
    }
}

impl KVStorage for MemoryDb {
    fn commit(&self, _column: Column, commit: Commit<TrieKey, Vec<u8>>) -> Result<()> {
        let mut data = self
            .data
            .write()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;

        for (key, value) in commit.iset() {
            data.insert(*key, value.clone());
        }

        for key in commit.iremoval() {
            data.remove(key);
        }

        Ok(())
    }

    fn set(&self, _column: Column, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<()> {
        let mut data = self
            .data
            .write()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;

        data.insert(key.as_ref().as_state_key(), value.as_ref().to_vec());
        Ok(())
    }

    fn get(&self, _column: Column, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        let data = self
            .data
            .read()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;
        Ok(data.get(&key.as_ref().as_state_key()).cloned())
    }

    fn iter(&self, _column: Column) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        let data = self
            .data
            .read()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;

        // Clone all entries to avoid holding the lock during iteration
        let entries: Vec<(Vec<u8>, Vec<u8>)> =
            data.iter().map(|(k, v)| (k.to_vec(), v.clone())).collect();

        Ok(entries.into_iter().map(Ok))
    }

    fn prefix_iter(
        &self,
        _column: Column,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        let data = self
            .data
            .read()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;
        let prefix_bytes = prefix.as_ref();

        // Clone all matching entries to avoid holding the lock during iteration
        let matches: Vec<(Vec<u8>, Vec<u8>)> = data
            .iter()
            .filter(|(k, _)| k.starts_with(prefix_bytes))
            .map(|(k, v)| (k.to_vec(), v.clone()))
            .collect();

        Ok(matches.into_iter().map(Ok))
    }
}

impl MultiTreeStore for MemoryDb {
    fn insert_tree(&self, _column: Column, key: OpaqueHash, root: NewNode) -> Result<()> {
        let mut tries = self
            .tries
            .write()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;
        if tries.roots.contains_key(&key) {
            anyhow::bail!(
                "insert_tree: tree already present at key 0x{} — caller must dereference_tree first",
                hex::encode(key)
            );
        }
        let addr = tries.insert(NodeRef::New(root));
        tries.roots.insert(key, addr);
        Ok(())
    }

    fn dereference_tree(&self, _column: Column, key: OpaqueHash) -> Result<()> {
        let mut tries = self
            .tries
            .write()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;
        if let Some(addr) = tries.roots.remove(&key) {
            tries.dereference(addr);
        }
        Ok(())
    }

    fn get_root(
        &self,
        _column: Column,
        key: OpaqueHash,
    ) -> Result<Option<(Vec<u8>, Vec<NodeAddress>)>> {
        let tries = self
            .tries
            .read()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;
        let Some(&addr) = tries.roots.get(&key) else {
            return Ok(None);
        };
        Ok(tries
            .nodes
            .get(&addr)
            .map(|n| (n.data.clone(), n.children.clone())))
    }

    fn get_node(
        &self,
        _column: Column,
        address: NodeAddress,
    ) -> Result<Option<(Vec<u8>, Vec<NodeAddress>)>> {
        let tries = self
            .tries
            .read()
            .map_err(|_| anyhow::anyhow!("RwLock poisoned"))?;
        Ok(tries
            .nodes
            .get(&address)
            .map(|n| (n.data.clone(), n.children.clone())))
    }
}
