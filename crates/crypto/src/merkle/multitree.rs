//! Ref-counted Merkle tree primitives and the incremental builder.

use crate::{blake2b, merkle::trie31};
use anyhow::{anyhow, Result};

const ZERO_HASH: [u8; 32] = [0u8; 32];

/// Switch from sequential to parallel descent when a build step covers this
/// many keys or more. Mirrors `trie31::PARALLEL_THRESHOLD`.
const PARALLEL_THRESHOLD: usize = 64;

/// Address of a node already persisted in the store.
pub type NodeAddress = u64;

/// A persisted node: payload + child addresses.
pub type PersistedNode = (Vec<u8>, Vec<NodeAddress>);

/// Reference to a node when building a tree: either a fresh subtree to write,
/// or a pointer to an existing node to reuse.
#[derive(Debug, Clone)]
pub enum NodeRef {
    /// A fresh subtree.
    New(NewNode),
    /// A node already in the store, retained as-is.
    Existing(NodeAddress),
}

/// A newly-built node with its child references.
#[derive(Debug, Clone)]
pub struct NewNode {
    /// Encoded node payload (leaf or branch).
    pub data: Vec<u8>,
    /// Child references, in canonical order.
    pub children: Vec<NodeRef>,
}

/// Ref-counted Merkle tree storage. Implementors provide the IO surface;
/// the `apply` default impl owns the recursion, leaf/branch encoding, and
/// subtree-reuse logic.
pub trait MultiTree: Sync {
    /// Look up a node by its persisted address.
    fn get_node(&self, address: NodeAddress) -> Result<Option<PersistedNode>>;

    /// Look up the root node of the tree identified by `hash`.
    fn get_root(&self, hash: [u8; 32]) -> Result<Option<PersistedNode>>;

    /// Persist a new tree under `hash`, bumping refcounts on existing children.
    fn insert_tree(&self, hash: [u8; 32], root: NewNode) -> Result<()>;

    /// Release the tree at `hash`, GC'ing nodes whose refcount falls to zero.
    fn dereference_tree(&self, hash: [u8; 32]) -> Result<()>;

    /// Build a new tree from `new_state`, reusing unchanged subtrees of `prev_root`.
    fn apply(
        &self,
        prev_root: Option<[u8; 32]>,
        new_state: &[([u8; 31], &[u8])],
        dirty_keys: &[[u8; 31]],
    ) -> Result<[u8; 32]> {
        let prev_root_node = match prev_root {
            Some(h) if h != ZERO_HASH => self.get_root(h)?,
            _ => None,
        };

        let (root_ref, new_root) = self.build(prev_root_node, new_state, dirty_keys, 0)?;

        if matches!(prev_root, Some(p) if p == new_root) {
            return Ok(new_root);
        }

        if let Some(NodeRef::New(root_node)) = root_ref {
            self.insert_tree(new_root, root_node)?;
        }

        if let Some(prev) = prev_root.filter(|&p| p != ZERO_HASH && p != new_root) {
            self.dereference_tree(prev)?;
        }

        Ok(new_root)
    }

    fn build(
        &self,
        prev_node: Option<PersistedNode>,
        new_keys: &[([u8; 31], &[u8])],
        new_dirty: &[[u8; 31]],
        depth: usize,
    ) -> Result<(Option<NodeRef>, [u8; 32])> {
        if new_keys.is_empty() {
            return Ok((None, ZERO_HASH));
        }

        if new_keys.len() == 1 {
            let (k, v) = new_keys[0];
            let data = trie31::leaf(k, v).to_vec();
            let hash = blake2b(&data);
            return Ok((
                Some(NodeRef::New(NewNode {
                    data,
                    children: vec![],
                })),
                hash,
            ));
        }

        let mut buf = new_keys.to_vec();
        let key_mid = trie31::partition(&mut buf, depth);
        let (left_keys, right_keys) = buf.split_at(key_mid);
        let dirty_mid = trie31::split_at_bit(new_dirty, depth);
        let (left_dirty, right_dirty) = new_dirty.split_at(dirty_mid);
        let (prev_left, prev_right) = match &prev_node {
            Some((data, children)) if !trie31::is_leaf(data) => {
                trie31::split_branch_children(data, children).ok_or_else(|| {
                    anyhow!(
                        "trie node shape mismatch at depth {depth}: children.len()={}",
                        children.len()
                    )
                })?
            }
            _ => (None, None),
        };

        let left = || self.descend(prev_left, left_keys, left_dirty, depth + 1);
        let right = || self.descend(prev_right, right_keys, right_dirty, depth + 1);
        let (l, r) = if new_keys.len() >= PARALLEL_THRESHOLD {
            rayon::join(left, right)
        } else {
            (left(), right())
        };

        let (l_ref, l_hash) = l?;
        let (r_ref, r_hash) = r?;
        let data = trie31::branch(l_hash, r_hash).to_vec();
        let hash = blake2b(&data);
        let children: Vec<NodeRef> = [l_ref, r_ref].into_iter().flatten().collect();
        Ok((Some(NodeRef::New(NewNode { data, children })), hash))
    }

    fn descend(
        &self,
        prev_addr: Option<NodeAddress>,
        new_keys: &[([u8; 31], &[u8])],
        new_dirty: &[[u8; 31]],
        depth: usize,
    ) -> Result<(Option<NodeRef>, [u8; 32])> {
        if new_dirty.is_empty() {
            if let Some(addr) = prev_addr {
                let (data, _) = self
                    .get_node(addr)?
                    .ok_or_else(|| anyhow!("missing prev node at address {addr}"))?;
                let hash = blake2b(&data);
                return Ok((Some(NodeRef::Existing(addr)), hash));
            }
            if new_keys.is_empty() {
                return Ok((None, ZERO_HASH));
            }
        }

        let prev_node = match prev_addr {
            Some(addr) => self.get_node(addr)?,
            None => None,
        };
        self.build(prev_node, new_keys, new_dirty, depth)
    }
}

/// In-memory ref-counted node store. Used as the multitree backend for
/// `MemoryDb` and tests; mirrors the contract of parity-db's multitree column
/// without disk persistence.
#[derive(Default)]
pub struct MultiTreeMap {
    inner: parking_lot::Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    nodes: std::collections::HashMap<NodeAddress, NodeEntry>,
    roots: std::collections::HashMap<[u8; 32], NodeAddress>,
    next_addr: NodeAddress,
}

struct NodeEntry {
    data: Vec<u8>,
    children: Vec<NodeAddress>,
    refcount: u32,
}

impl Inner {
    fn alloc(&mut self) -> NodeAddress {
        let addr = self.next_addr;
        self.next_addr += 1;
        addr
    }

    fn insert(&mut self, node_ref: NodeRef) -> anyhow::Result<NodeAddress> {
        match node_ref {
            NodeRef::Existing(addr) => {
                let Some(entry) = self.nodes.get_mut(&addr) else {
                    anyhow::bail!("NodeRef::Existing({addr}) but node is gone");
                };

                entry.refcount += 1;
                Ok(addr)
            }
            NodeRef::New(NewNode { data, children }) => {
                let child_addrs: Vec<_> = children
                    .into_iter()
                    .map(|c| self.insert(c))
                    .collect::<Result<Vec<_>>>()?;
                let addr = self.alloc();
                self.nodes.insert(
                    addr,
                    NodeEntry {
                        data,
                        children: child_addrs,
                        refcount: 1,
                    },
                );
                Ok(addr)
            }
        }
    }

    fn dereference(&mut self, addr: NodeAddress) -> anyhow::Result<()> {
        let drop_children = match self.nodes.get_mut(&addr) {
            Some(entry) => {
                entry.refcount = entry
                    .refcount
                    .checked_sub(1)
                    .ok_or_else(|| anyhow!("refcount underflow on addr {addr}"))?;
                (entry.refcount == 0).then(|| std::mem::take(&mut entry.children))
            }
            None => None,
        };
        if let Some(children) = drop_children {
            self.nodes.remove(&addr);
            for child in children {
                self.dereference(child)?;
            }
        }

        Ok(())
    }
}

impl MultiTree for MultiTreeMap {
    fn insert_tree(&self, key: [u8; 32], root: NewNode) -> Result<()> {
        let mut inner = self.inner.lock();
        if inner.roots.contains_key(&key) {
            anyhow::bail!("insert_tree: tree already present — caller must dereference_tree first");
        }
        let addr = inner.insert(NodeRef::New(root))?;
        inner.roots.insert(key, addr);
        Ok(())
    }

    fn dereference_tree(&self, key: [u8; 32]) -> Result<()> {
        let mut inner = self.inner.lock();
        if let Some(addr) = inner.roots.remove(&key) {
            inner.dereference(addr)?;
        }
        Ok(())
    }

    fn get_root(&self, key: [u8; 32]) -> Result<Option<PersistedNode>> {
        let inner = self.inner.lock();
        let Some(&addr) = inner.roots.get(&key) else {
            return Ok(None);
        };
        Ok(inner
            .nodes
            .get(&addr)
            .map(|n| (n.data.clone(), n.children.clone())))
    }

    fn get_node(&self, address: NodeAddress) -> Result<Option<PersistedNode>> {
        Ok(self
            .inner
            .lock()
            .nodes
            .get(&address)
            .map(|n| (n.data.clone(), n.children.clone())))
    }
}
